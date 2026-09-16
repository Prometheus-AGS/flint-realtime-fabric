#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{
    ChannelId, CommittedEntityMutation, EntityTypeDelivery, EventEnvelope, EventKind, Offset,
};
use frf_ports::{EntityTypeDeliveryStream, EntityTypeWatchSource, LogBroker, PortError};
use futures_util::StreamExt as _;
use tracing::instrument;

/// Converts the well-known committed entity channel into typed watch deliveries.
pub struct BrokerEntityTypeWatchSource<L> {
    broker: Arc<L>,
}

impl<L> BrokerEntityTypeWatchSource<L> {
    #[must_use]
    pub fn new(broker: Arc<L>) -> Self {
        Self { broker }
    }
}

#[async_trait]
impl<L> EntityTypeWatchSource for BrokerEntityTypeWatchSource<L>
where
    L: LogBroker,
{
    #[instrument(name = "port::EntityTypeWatchSource::subscribe", skip(self))]
    async fn subscribe(
        &self,
        consumer_id: String,
        from: Offset,
    ) -> Result<EntityTypeDeliveryStream, PortError> {
        let stream = self
            .broker
            .subscribe(ChannelId::WELL_KNOWN_ENTITIES, consumer_id, from)
            .await?;
        Ok(Box::pin(stream.map(|item| item.and_then(decode_delivery))))
    }

    #[instrument(name = "port::EntityTypeWatchSource::head_offset", skip(self))]
    async fn head_offset(&self) -> Result<Option<Offset>, PortError> {
        self.broker
            .head_offset(ChannelId::WELL_KNOWN_ENTITIES)
            .await
    }
}

fn decode_delivery(envelope: EventEnvelope) -> Result<EntityTypeDelivery, PortError> {
    if envelope.kind != EventKind::EntityChange {
        return Err(PortError::Serialization(
            "entity watch channel carried a non-entity event".to_owned(),
        ));
    }
    if envelope.offset == Offset::BEGINNING && envelope.payload.is_null() {
        return Err(PortError::Serialization(
            "entity watch delivery is empty".to_owned(),
        ));
    }
    let mutation: CommittedEntityMutation = serde_json::from_value(envelope.payload)
        .map_err(|error| PortError::Serialization(error.to_string()))?;
    // This is an envelope-integrity check. It does not compare against a
    // subscriber tenant; valid events from every tenant continue to the app.
    if mutation.tenant_id != envelope.channel.tenant_id {
        return Err(PortError::PermissionDenied(
            "entity watch envelope tenant mismatch".to_owned(),
        ));
    }
    Ok(EntityTypeDelivery {
        mutation,
        broker_partition: 0,
        broker_offset: envelope.offset.0,
    })
}

#[cfg(test)]
mod tests {
    use frf_domain::{Channel, TenantId};

    use super::*;

    fn envelope(channel_tenant: TenantId, mutation_tenant: TenantId) -> EventEnvelope {
        EventEnvelope::new(
            Channel {
                id: ChannelId::WELL_KNOWN_ENTITIES,
                tenant_id: channel_tenant,
                path: "entities".to_owned(),
            },
            Offset(7),
            EventKind::EntityChange,
            serde_json::json!({
                "event_id": "frfevent:v1:test",
                "schema": "public",
                "table": "orders",
                "projection": "default",
                "entity_type": "public.orders@default",
                "tenant_id": mutation_tenant,
                "key": {"parts": [], "canonical_id": "frfkey:v1:1"},
                "op": "insert",
                "record": [],
                "unchanged_toast": [],
                "source": {"epoch": "test", "commit_lsn": 1, "transaction_index": 0},
                "committed_at": "2026-09-16T00:00:00Z"
            }),
        )
    }

    #[test]
    fn valid_events_from_multiple_tenants_reach_the_application_filter() {
        let first = TenantId::new();
        let second = TenantId::new();
        assert_eq!(
            decode_delivery(envelope(first, first))
                .unwrap()
                .mutation
                .tenant_id,
            first
        );
        assert_eq!(
            decode_delivery(envelope(second, second))
                .unwrap()
                .mutation
                .tenant_id,
            second
        );
    }

    #[test]
    fn inconsistent_envelope_and_mutation_tenants_fail_closed() {
        let error = decode_delivery(envelope(TenantId::new(), TenantId::new())).unwrap_err();
        assert!(matches!(error, PortError::PermissionDenied(_)));
    }
}
