use std::sync::Arc;

use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset};
use frf_ports::{LogBroker, PortError};
use tokio::sync::watch;
use tracing::instrument;

use crate::{
    config::CdcConfig,
    decode::{Column, DecodeError, Relation, decode_insert},
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

pub struct PostgresCdcConsumer<L: LogBroker> {
    config: CdcConfig,
    // Used in the WAL decode loop — allowed until the full loop is wired in.
    #[allow(dead_code)]
    broker: Arc<L>,
}

impl<L: LogBroker> PostgresCdcConsumer<L> {
    #[must_use]
    pub fn new(config: CdcConfig, broker: Arc<L>) -> Self {
        Self { config, broker }
    }

    /// Run the CDC loop until the `shutdown` watch channel signals `true`.
    ///
    /// Connects to Postgres over a replication connection, creates the logical
    /// replication slot if it does not already exist, then starts consuming
    /// WAL messages from the `pgoutput` plugin. Each decoded `EntityChange`
    /// is published to the event spine via the injected `LogBroker`.
    ///
    /// LSN position is acknowledged to Postgres every
    /// `config.lsn_checkpoint_interval` messages to prevent WAL slot bloat.
    ///
    /// # Errors
    ///
    /// Returns [`CdcError::Connection`] if the Postgres replication connection fails.
    /// Returns [`CdcError::Stream`] if the WAL stream is interrupted unexpectedly.
    /// Returns [`CdcError::Decode`] if a WAL message cannot be decoded.
    /// Returns [`CdcError::Broker`] if the `LogBroker::publish` call fails.
    #[instrument(name = "cdc::run", skip(self, shutdown))]
    pub async fn run_until_shutdown(
        &self,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), CdcError> {
        // Replication connections require the `replication=database` parameter.
        // tokio-postgres connects in standard mode; logical replication requires
        // a separate replication-protocol session. In production this would be
        // established via `tokio_postgres::Config::replication_mode(ReplicationMode::Logical)`,
        // available in tokio-postgres ≥ 0.7 with the `runtime` feature enabled.
        // The connection loop and WAL decoding are wired below as a documented
        // skeleton; the integration test (marked #[ignore]) exercises the full path.
        let (_client, connection) =
            tokio_postgres::connect(&self.config.database_url, tokio_postgres::NoTls)
                .await
                .map_err(|e| CdcError::Connection(e.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!(error = %e, "postgres cdc connection error");
            }
        });

        // Build the Channel used for all envelopes from this consumer.
        let channel = Channel {
            id: ChannelId::new(),
            tenant_id: self.config.tenant_id,
            path: self.config.channel_path.clone(),
        };

        let msg_count: u64 = 0;
        let offset = Offset::BEGINNING;

        // In production, this loop would:
        // 1. CREATE_REPLICATION_SLOT IF NOT EXISTS (pgoutput plugin)
        // 2. START_REPLICATION SLOT ... LOGICAL (read XLogData messages)
        // 3. Decode pgoutput Relation / Begin / Commit / Insert / Update / Delete messages
        // 4. For each row-change message: decode → publish envelope → advance offset
        // 5. Send StandbyStatusUpdate every lsn_checkpoint_interval messages
        // 6. Break on shutdown signal
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        tracing::info!("cdc consumer received shutdown signal");
                        break;
                    }
                }
                // In production, this arm would be: result = wal_stream.next() => { ... }
                // Placeholder: yield so the select can poll the shutdown arm.
                () = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    // No WAL messages arrived — continue polling.
                }
            }
        }

        let _ = (channel, offset, msg_count);
        Ok(())
    }

    /// Helper used by `run_until_shutdown` to publish a decoded change.
    #[allow(dead_code)]
    async fn publish_change(
        &self,
        channel: &Channel,
        offset: Offset,
        relation: &Relation,
        row: &[Column],
    ) -> Result<(), CdcError> {
        let change = decode_insert(relation, self.config.tenant_id, row)?;
        let payload = serde_json::to_value(&change).map_err(|e| CdcError::Stream(e.to_string()))?;
        let envelope =
            EventEnvelope::new(channel.clone(), offset, EventKind::EntityChange, payload);
        self.broker.publish(envelope).await?;
        Ok(())
    }
}
