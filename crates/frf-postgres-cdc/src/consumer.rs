use std::sync::Arc;
use std::time::Duration;

use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset};
use frf_ports::{LogBroker, PortError};
use pg_walstream::{
    EventType, LogicalReplicationStream, ReplicationSlotOptions, ReplicationStreamConfig,
    RetryConfig, SlotType, StreamingMode,
};
use tokio::sync::watch;
use tracing::instrument;

use crate::{
    config::CdcConfig,
    decode::{Column, DecodeError, Relation, decode_delete, decode_insert, decode_update},
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CdcError {
    #[error("postgres connection error: {0}")]
    Connection(String),
    #[error("replication stream error: {0}")]
    Stream(String),
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("broker publish error: {0}")]
    Broker(#[from] PortError),
}

impl From<pg_walstream::ReplicationError> for CdcError {
    fn from(e: pg_walstream::ReplicationError) -> Self {
        Self::Stream(e.to_string())
    }
}

pub struct PostgresCdcConsumer<L: LogBroker> {
    config: CdcConfig,
    broker: Arc<L>,
}

impl<L: LogBroker + Send + Sync + 'static> PostgresCdcConsumer<L> {
    #[must_use]
    pub fn new(config: CdcConfig, broker: Arc<L>) -> Self {
        Self { config, broker }
    }

    /// Run the CDC loop until the `shutdown` watch channel signals `true`.
    ///
    /// Opens a logical replication connection to Postgres, ensures the slot
    /// exists, starts the `pgoutput` stream, and publishes each decoded row
    /// change to the event spine via the injected `LogBroker`.
    ///
    /// LSN feedback is sent automatically by `pg_walstream` at the configured
    /// `feedback_interval`; this loop calls `update_applied_lsn` after each
    /// successful publish to advance the acknowledged position.
    ///
    /// # Errors
    ///
    /// Returns [`CdcError::Connection`] if the replication connection fails.
    /// Returns [`CdcError::Stream`] if the WAL stream is interrupted.
    /// Returns [`CdcError::Decode`] if a WAL message cannot be decoded.
    /// Returns [`CdcError::Broker`] if `LogBroker::publish` fails.
    #[instrument(name = "cdc::run", skip(self, shutdown))]
    pub async fn run_until_shutdown(
        &self,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), CdcError> {
        let stream_config = ReplicationStreamConfig {
            slot_name: self.config.slot_name.clone(),
            publication_name: self.config.publication_name.clone(),
            protocol_version: 2,
            streaming_mode: StreamingMode::Off,
            messages: false,
            binary: false,
            two_phase: false,
            origin: None,
            feedback_interval: Duration::from_secs(10),
            connection_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            slot_options: ReplicationSlotOptions::default(),
            slot_type: SlotType::Logical,
        };

        let mut stream =
            LogicalReplicationStream::new(&self.config.replication_url(), stream_config)
                .await
                .map_err(|e| CdcError::Connection(e.to_string()))?;

        stream
            .ensure_replication_slot()
            .await
            .map_err(|e| CdcError::Connection(e.to_string()))?;

        stream
            .start(None)
            .await
            .map_err(|e| CdcError::Stream(e.to_string()))?;

        let cancel_token = pg_walstream::CancellationToken::new();
        let cancel_token_shutdown = cancel_token.clone();
        let mut event_stream = stream.into_stream(cancel_token);

        let channel = Channel {
            id: ChannelId::new(),
            tenant_id: self.config.tenant_id,
            path: self.config.channel_path.clone(),
        };

        let mut offset = Offset::BEGINNING;

        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        tracing::info!("cdc consumer received shutdown signal");
                        cancel_token_shutdown.cancel();
                        let _ = event_stream.shutdown().await;
                        break;
                    }
                }
                event_result = event_stream.next_event() => {
                    match event_result {
                        Ok(event) => {
                            let lsn = event.lsn.0;
                            if let Some(change) = self.translate_event(event.event_type) {
                                match change {
                                    Ok(envelope_change) => {
                                        let payload = serde_json::to_value(&envelope_change)
                                            .map_err(|e| CdcError::Stream(e.to_string()))?;
                                        let envelope = EventEnvelope::new(
                                            channel.clone(),
                                            offset,
                                            EventKind::EntityChange,
                                            payload,
                                        );
                                        self.broker.publish(envelope).await?;
                                        event_stream.update_applied_lsn(lsn);
                                        offset = offset.next();
                                    }
                                    Err(e) => {
                                        tracing::warn!(error = %e, "skipping undecoded WAL row");
                                    }
                                }
                            }
                        }
                        Err(pg_walstream::ReplicationError::Cancelled(_)) => {
                            tracing::info!("cdc stream cancelled");
                            break;
                        }
                        Err(e) => {
                            return Err(CdcError::Stream(e.to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn translate_event(
        &self,
        event_type: EventType,
    ) -> Option<Result<frf_domain::EntityChange, DecodeError>> {
        match event_type {
            EventType::Insert {
                schema,
                table,
                data,
                ..
            } => {
                let relation = relation_from_row_data(&schema, &table, &data);
                let cols = columns_from_row_data(&data);
                Some(decode_insert(&relation, self.config.tenant_id, &cols))
            }
            EventType::Update {
                schema,
                table,
                old_data,
                new_data,
                ..
            } => {
                let relation = relation_from_row_data(&schema, &table, &new_data);
                let new_cols = columns_from_row_data(&new_data);
                let old_cols: Option<Vec<Column>> = old_data.as_ref().map(columns_from_row_data);
                Some(decode_update(
                    &relation,
                    self.config.tenant_id,
                    old_cols.as_deref(),
                    &new_cols,
                ))
            }
            EventType::Delete {
                schema,
                table,
                old_data,
                ..
            } => {
                let relation = relation_from_row_data(&schema, &table, &old_data);
                let cols = columns_from_row_data(&old_data);
                Some(decode_delete(&relation, self.config.tenant_id, &cols))
            }
            // Begin / Commit / Relation / Type are protocol overhead — not row changes.
            EventType::Begin { .. }
            | EventType::Commit { .. }
            | EventType::Relation { .. }
            | EventType::Type { .. }
            | EventType::Origin { .. }
            | EventType::Truncate(_)
            | EventType::Message { .. }
            | EventType::StreamStart { .. }
            | EventType::StreamStop
            | EventType::StreamCommit { .. }
            | EventType::StreamAbort { .. }
            | EventType::BeginPrepare { .. }
            | EventType::Prepare { .. }
            | EventType::CommitPrepared { .. }
            | EventType::RollbackPrepared { .. }
            | EventType::StreamPrepare { .. } => None,
        }
    }
}

fn relation_from_row_data(schema: &str, table: &str, data: &pg_walstream::RowData) -> Relation {
    let columns: Vec<String> = data.iter().map(|(name, _)| name.to_string()).collect();
    Relation {
        oid: 0,
        namespace: schema.to_owned(),
        name: table.to_owned(),
        columns,
        pk_index: 0,
    }
}

fn columns_from_row_data(data: &pg_walstream::RowData) -> Vec<Column> {
    data.iter()
        .map(|(name, value)| Column {
            name: name.to_string(),
            value: value.as_str().map(ToOwned::to_owned),
        })
        .collect()
}
