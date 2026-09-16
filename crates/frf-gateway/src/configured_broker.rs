use async_trait::async_trait;
use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset};
use frf_ports::{EventStream, LogBroker, PortError};

/// Runtime broker selected by the deployment profile.
///
/// The full profile owns a durable Iggy adapter. The shape-only profile uses
/// the disabled variant so it cannot connect to, publish to, or subscribe from
/// an event spine that is outside its declared authority.
pub(crate) enum ConfiguredLogBroker {
    Iggy(IggyBroker),
    Disabled,
}

impl ConfiguredLogBroker {
    pub(crate) async fn for_full(
        connection_string: &str,
        replay_retention_seconds: u64,
    ) -> anyhow::Result<Self> {
        Ok(Self::Iggy(
            IggyBroker::with_replay_retention(connection_string, replay_retention_seconds).await?,
        ))
    }

    const fn disabled() -> Self {
        Self::Disabled
    }

    pub(crate) fn for_profile(profile: frf_gateway::GatewayProfile) -> Option<Self> {
        (profile == frf_gateway::GatewayProfile::ShapeOnly).then(Self::disabled)
    }

    fn disabled_error() -> PortError {
        PortError::PermissionDenied(
            "event spine is disabled for the shape-only gateway profile".to_owned(),
        )
    }
}

#[async_trait]
impl LogBroker for ConfiguredLogBroker {
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        match self {
            Self::Iggy(broker) => broker.publish(envelope).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }

    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError> {
        match self {
            Self::Iggy(broker) => broker.subscribe(channel_id, consumer_id, from).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }

    async fn head_offset(&self, channel_id: ChannelId) -> Result<Option<Offset>, PortError> {
        match self {
            Self::Iggy(broker) => broker.head_offset(channel_id).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }

    async fn seek(&self, cursor: Cursor) -> Result<(), PortError> {
        match self {
            Self::Iggy(broker) => broker.seek(cursor).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }

    async fn ack(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
        offset: Offset,
    ) -> Result<(), PortError> {
        match self {
            Self::Iggy(broker) => broker.ack(channel_id, consumer_id, offset).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }

    async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError> {
        match self {
            Self::Iggy(broker) => broker.ensure_channel(channel).await,
            Self::Disabled => Err(Self::disabled_error()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shape_only_broker_rejects_event_operations() {
        let broker = ConfiguredLogBroker::for_profile(frf_gateway::GatewayProfile::ShapeOnly)
            .expect("shape-only broker");
        let error = broker
            .ensure_channel(Channel {
                id: ChannelId::WELL_KNOWN_ENTITIES,
                tenant_id: frf_domain::TenantId::from_uuid(uuid::Uuid::nil()),
                path: "entities".into(),
            })
            .await
            .expect_err("shape-only must reject broker operations");
        assert!(error.to_string().contains("shape-only"));
        let error = broker
            .head_offset(ChannelId::WELL_KNOWN_ENTITIES)
            .await
            .expect_err("shape-only must reject high-water queries");
        assert!(error.to_string().contains("shape-only"));
    }
}
