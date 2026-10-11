use std::sync::Arc;
use std::time::Duration;

use frf_domain::ids::EventId;
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset};
use frf_ports::{LogBroker, PortError};
use pg_walstream::{
    LogicalReplicationStream, ReplicationSlotOptions, ReplicationStreamConfig, RetryConfig,
    SlotType, StreamingMode,
};
use tokio::sync::watch;
use tracing::instrument;

use crate::{
    catalog::{Catalog, CatalogError},
    config::CdcConfig,
    transaction::{CommittedTransaction, TransactionAssembler, TransactionError},
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CdcError {
    #[error("postgres connection error: {0}")]
    Connection(String),
    #[error("replication stream error: {0}")]
    Stream(String),
    #[error("CDC enrollment error: {0}")]
    Enrollment(#[from] CatalogError),
    #[error("CDC transaction is poisoned: {0}")]
    Transaction(#[from] TransactionError),
    #[error("broker publish error: {0}")]
    Broker(#[from] PortError),
}

impl From<pg_walstream::ReplicationError> for CdcError {
    fn from(error: pg_walstream::ReplicationError) -> Self {
        Self::Stream(error.to_string())
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
    /// Enrollment is validated against the live PostgreSQL catalog before the
    /// replication stream starts. Row changes are buffered until COMMIT, then
    /// durably published in transaction order. Applied-LSN feedback advances
    /// only after every mutation in that commit was accepted by the broker.
    ///
    /// # Errors
    ///
    /// Returns [`CdcError`] when enrollment, replication, decoding or durable
    /// publication fails. A poison transaction stops without advancing its LSN.
    #[instrument(name = "cdc::run", skip(self, shutdown))]
    pub async fn run_until_shutdown(
        &self,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(), CdcError> {
        self.run(shutdown, None).await
    }

    /// Run the CDC loop and publish whether logical replication is active.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::run_until_shutdown`].
    pub async fn run_until_shutdown_with_readiness(
        &self,
        shutdown: watch::Receiver<bool>,
        readiness: watch::Sender<bool>,
    ) -> Result<(), CdcError> {
        self.run(shutdown, Some(readiness)).await
    }

    async fn run(
        &self,
        mut shutdown: watch::Receiver<bool>,
        readiness: Option<watch::Sender<bool>>,
    ) -> Result<(), CdcError> {
        let readiness = ReadinessGuard::new(readiness);
        let catalog = Catalog::load(&self.config).await?;
        let epoch = self
            .config
            .source_epoch
            .as_deref()
            .ok_or(CatalogError::InvalidEpoch)?;
        let mut stream = LogicalReplicationStream::new(
            &self.config.replication_url(),
            replication_config(&self.config),
        )
        .await
        .map_err(|error| CdcError::Connection(error.to_string()))?;
        stream
            .ensure_replication_slot()
            .await
            .map_err(|error| CdcError::Connection(error.to_string()))?;
        stream
            .start(None)
            .await
            .map_err(|error| CdcError::Stream(error.to_string()))?;

        let cancel_token = pg_walstream::CancellationToken::new();
        let shutdown_token = cancel_token.clone();
        let mut event_stream = stream.into_stream(cancel_token);
        let channel = Channel {
            id: ChannelId::WELL_KNOWN_ENTITIES,
            tenant_id: self.config.tenant_id,
            path: self.config.channel_path.clone(),
        };
        let mut transactions = TransactionAssembler::from_catalog(&catalog);
        let mut messages_since_feedback = 0_u64;
        readiness.mark_ready();
        tracing::info!(
            channel_id = %channel.id,
            path = %channel.path,
            "CDC transaction consumer is ready",
        );

        loop {
            tokio::select! {
                biased;
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        tracing::info!("CDC consumer received shutdown signal");
                        shutdown_token.cancel();
                        event_stream.shutdown().await?;
                        break;
                    }
                }
                event_result = event_stream.next_event() => {
                    match event_result {
                        Ok(event) => {
                            if let Some(commit) = transactions.accept(
                                event.event_type,
                                &catalog,
                                self.config.tenant_id,
                                epoch,
                            )? {
                                catalog.revalidate(&self.config).await?;
                                self.publish_commit(&channel, &commit).await?;
                                event_stream.update_applied_lsn(commit.end_lsn);
                            }
                            messages_since_feedback = messages_since_feedback.saturating_add(1);
                            if messages_since_feedback >= self.config.lsn_checkpoint_interval {
                                event_stream.inner_mut().send_feedback().await?;
                                messages_since_feedback = 0;
                            }
                        }
                        Err(pg_walstream::ReplicationError::Cancelled(_)) => {
                            tracing::info!("CDC stream cancelled");
                            break;
                        }
                        Err(error) => return Err(CdcError::Stream(error.to_string())),
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_commit(
        &self,
        channel: &Channel,
        commit: &CommittedTransaction,
    ) -> Result<(), CdcError> {
        for (mutation, envelope_uuid) in &commit.mutations {
            let payload = serde_json::to_value(mutation)
                .map_err(|error| CdcError::Stream(error.to_string()))?;
            let mut event_channel = channel.clone();
            event_channel.tenant_id = mutation.tenant_id;
            self.broker.ensure_channel(event_channel.clone()).await?;
            let envelope = EventEnvelope {
                id: EventId::from_uuid(*envelope_uuid),
                channel: event_channel,
                // The LogBroker assigns the durable partition position. The
                // complete source position remains in `mutation.source`.
                offset: Offset::BEGINNING,
                kind: EventKind::EntityChange,
                payload,
                timestamp: mutation.committed_at,
                correlation_id: Some(mutation.event_id.clone()),
            };
            let broker_offset = self.broker.publish(envelope).await?;
            tracing::debug!(
                broker_offset = broker_offset.0,
                "published committed CDC mutation",
            );
        }
        Ok(())
    }
}

fn replication_config(config: &CdcConfig) -> ReplicationStreamConfig {
    ReplicationStreamConfig {
        slot_name: config.slot_name.clone(),
        publication_name: config.publication_name.clone(),
        protocol_version: 2,
        streaming_mode: StreamingMode::Off,
        messages: false,
        binary: false,
        two_phase: false,
        origin: None,
        feedback_interval: Duration::from_secs(1),
        connection_timeout: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(30),
        retry_config: RetryConfig::default(),
        slot_options: ReplicationSlotOptions::default(),
        slot_type: SlotType::Logical,
    }
}

struct ReadinessGuard(Option<watch::Sender<bool>>);

impl ReadinessGuard {
    fn new(sender: Option<watch::Sender<bool>>) -> Self {
        if let Some(sender) = &sender {
            sender.send_replace(false);
        }
        Self(sender)
    }

    fn mark_ready(&self) {
        if let Some(sender) = &self.0 {
            sender.send_replace(true);
        }
    }
}

impl Drop for ReadinessGuard {
    fn drop(&mut self) {
        if let Some(sender) = &self.0 {
            sender.send_replace(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReadinessGuard;

    #[test]
    fn readiness_guard_is_false_before_start_and_after_exit() {
        let (sender, receiver) = tokio::sync::watch::channel(true);
        {
            let guard = ReadinessGuard::new(Some(sender));
            assert!(!*receiver.borrow());
            guard.mark_ready();
            assert!(*receiver.borrow());
        }
        assert!(!*receiver.borrow());
    }
}
