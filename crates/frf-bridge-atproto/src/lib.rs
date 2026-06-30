#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod convert;
pub mod error;
pub mod jetstream;

use async_trait::async_trait;
use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{
    error::PortError,
    federation::{FederationBridge, FederationProtocol, FederationStream},
};
use tracing::instrument;

/// `FederationBridge` adapter for the `ATProto` / Bluesky Jetstream protocol.
///
/// Connects to the Bluesky public Jetstream WebSocket API and projects commit
/// events onto the FRF spine as `FederatedEvent`s.
///
/// The write path returns `Err(PortError::Transport(...))` — `ATProto` write
/// operations require PDS authentication and are deferred to a future phase.
pub struct AtProtoBridge {
    jetstream_url: String,
    collections: Vec<String>,
    tenant_id: TenantId,
    channel_id: ChannelId,
}

impl AtProtoBridge {
    /// Create a new `AtProtoBridge` that consumes from `jetstream_url`.
    ///
    /// `collections` filters events by `ATProto` lexicon type, e.g.
    /// `vec!["app.bsky.feed.post".to_owned()]`.
    #[must_use]
    pub fn new(
        jetstream_url: impl Into<String>,
        collections: Vec<String>,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            jetstream_url: jetstream_url.into(),
            collections,
            tenant_id,
            channel_id,
        }
    }
}

#[async_trait]
impl FederationBridge for AtProtoBridge {
    #[instrument(skip(self, _envelope, _destination), fields(protocol = ?protocol))]
    async fn send(
        &self,
        protocol: FederationProtocol,
        _destination: &str,
        _envelope: frf_domain::EventEnvelope,
    ) -> Result<(), PortError> {
        Err(PortError::Transport(format!(
            "AtProtoBridge: write path not implemented for {protocol:?}"
        )))
    }

    #[instrument(skip(self), fields(protocol = ?protocol))]
    async fn subscribe(&self, protocol: FederationProtocol) -> Result<FederationStream, PortError> {
        if protocol != FederationProtocol::AtProto {
            return Err(PortError::Transport(format!(
                "AtProtoBridge cannot subscribe to {protocol:?} events"
            )));
        }

        let stream = jetstream::jetstream_stream(
            self.jetstream_url.clone(),
            self.collections.clone(),
            self.tenant_id,
            self.channel_id,
        );

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bridge() -> AtProtoBridge {
        AtProtoBridge::new(
            "wss://jetstream2.us-east.bsky.network/subscribe",
            vec!["app.bsky.feed.post".to_owned()],
            TenantId::new(),
            ChannelId::new(),
        )
    }

    #[tokio::test]
    async fn subscribe_returns_stream_for_atproto() {
        let bridge = make_bridge();
        // subscribe() should succeed — the stream itself may not yield events
        // in a unit test environment without a live Jetstream connection.
        let result = bridge.subscribe(FederationProtocol::AtProto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn subscribe_rejects_wrong_protocol() {
        let bridge = make_bridge();
        let result = bridge.subscribe(FederationProtocol::Matrix).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn send_returns_unsupported_error() {
        use frf_domain::envelope::{Channel, EventEnvelope, EventKind, Offset};
        use frf_domain::ids::ChannelId as CId;

        let bridge = make_bridge();
        let envelope = EventEnvelope::new(
            Channel {
                id: CId::new(),
                tenant_id: TenantId::new(),
                path: "test".to_owned(),
            },
            Offset::BEGINNING,
            EventKind::Custom("test".to_owned()),
            serde_json::json!({}),
        );

        let result = bridge
            .send(FederationProtocol::AtProto, "dest", envelope)
            .await;
        assert!(result.is_err());
    }
}
