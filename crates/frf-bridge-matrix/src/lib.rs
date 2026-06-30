#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod convert;
pub mod error;

use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{
    error::PortError,
    federation::{FederationBridge, FederationProtocol, FederationStream},
};
use tracing::instrument;

use crate::client::MatrixClient;

/// `FederationBridge` adapter for the Matrix federation protocol.
///
/// Uses a [`MatrixClient`] abstraction over the homeserver connection. Provide
/// a [`ReqwestMatrixClient`](client::ReqwestMatrixClient) for production or a
/// [`MockMatrixClient`](client::MockMatrixClient) for tests.
pub struct MatrixBridge {
    client: Arc<dyn MatrixClient + Send + Sync>,
    room_id: String,
    tenant_id: TenantId,
    channel_id: ChannelId,
}

impl MatrixBridge {
    /// Create a new `MatrixBridge` for the given room.
    #[must_use]
    pub fn new(
        client: impl MatrixClient + 'static,
        room_id: impl Into<String>,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            client: Arc::new(client),
            room_id: room_id.into(),
            tenant_id,
            channel_id,
        }
    }
}

#[async_trait]
impl FederationBridge for MatrixBridge {
    #[instrument(skip(self, envelope), fields(destination, protocol = ?protocol))]
    async fn send(
        &self,
        protocol: FederationProtocol,
        destination: &str,
        envelope: frf_domain::EventEnvelope,
    ) -> Result<(), PortError> {
        if protocol != FederationProtocol::Matrix {
            return Err(PortError::Transport(format!(
                "MatrixBridge cannot send {protocol:?} events"
            )));
        }

        let content = serde_json::json!({
            "msgtype": "m.text",
            "destination": destination,
            "body": serde_json::to_string(&envelope.payload)
                .unwrap_or_else(|_| String::from("<unserializable>")),
        });

        self.client
            .send_event(&self.room_id, content)
            .await
            .map_err(PortError::from)
    }

    #[instrument(skip(self), fields(protocol = ?protocol))]
    async fn subscribe(&self, protocol: FederationProtocol) -> Result<FederationStream, PortError> {
        if protocol != FederationProtocol::Matrix {
            return Err(PortError::Transport(format!(
                "MatrixBridge cannot subscribe to {protocol:?} events"
            )));
        }

        let stream =
            self.client
                .room_event_stream(self.room_id.clone(), self.tenant_id, self.channel_id);

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{MockMatrixClient, RawMatrixEvent};
    use futures_util::StreamExt;

    fn make_bridge() -> MatrixBridge {
        let mock = MockMatrixClient {
            events: vec![RawMatrixEvent {
                event_id: Some("$test-event".to_owned()),
                sender: Some("@bot:matrix.org".to_owned()),
                content: serde_json::json!({"msgtype": "m.text", "body": "hello"}),
            }],
        };

        MatrixBridge::new(
            mock,
            "!test-room:matrix.org",
            TenantId::new(),
            ChannelId::new(),
        )
    }

    #[tokio::test]
    async fn subscribe_yields_projected_events() {
        let bridge = make_bridge();
        let mut stream = bridge
            .subscribe(FederationProtocol::Matrix)
            .await
            .expect("subscribe should succeed");

        let event = stream
            .next()
            .await
            .expect("should yield one event")
            .unwrap();

        assert_eq!(event.protocol, FederationProtocol::Matrix);
        assert!(event.source.contains("$test-event"));
    }

    #[tokio::test]
    async fn subscribe_rejects_wrong_protocol() {
        let bridge = make_bridge();
        let result = bridge.subscribe(FederationProtocol::AtProto).await;
        assert!(result.is_err());
    }
}
