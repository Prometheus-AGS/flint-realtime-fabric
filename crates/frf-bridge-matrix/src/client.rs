use async_trait::async_trait;
use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{error::PortError, federation::FederatedEvent};
use futures_util::stream::{self, BoxStream};

use crate::error::MatrixBridgeError;

/// A raw Matrix room event as returned by the homeserver client API.
#[derive(Debug, Clone)]
pub struct RawMatrixEvent {
    pub event_id: Option<String>,
    pub sender: Option<String>,
    pub content: serde_json::Value,
}

/// Abstraction over Matrix homeserver client implementations.
///
/// Implementations: `ReqwestMatrixClient` (REST stub), `MockMatrixClient` (tests).
#[async_trait]
pub trait MatrixClient: Send + Sync {
    /// Stream room events from the given room ID.
    fn room_event_stream(
        &self,
        room_id: String,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> BoxStream<'static, Result<FederatedEvent, PortError>>;

    /// Send a Matrix event to the given room.
    ///
    /// # Errors
    ///
    /// Returns a [`MatrixBridgeError`] if the request fails.
    async fn send_event(
        &self,
        room_id: &str,
        content: serde_json::Value,
    ) -> Result<(), MatrixBridgeError>;
}

/// REST-based Matrix client using the Client-Server API.
///
/// This stub polls `/sync` for room events. In production, replace with a
/// persistent `tokio-tungstenite` WebSocket connection or a Tuwunel library dep.
///
/// BLOCKED_ON_TUWUNEL: Tuwunel does not yet expose a stable Rust library crate.
/// Track <https://github.com/girlbossceo/tuwunel/issues> for crate publication.
/// Replace this REST stub with a native Tuwunel client once the crate is available.
pub struct ReqwestMatrixClient {
    http: reqwest::Client,
    homeserver_url: String,
    access_token: String,
}

impl ReqwestMatrixClient {
    /// Create a new REST Matrix client.
    #[must_use]
    pub fn new(homeserver_url: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            homeserver_url: homeserver_url.into(),
            access_token: access_token.into(),
        }
    }
}

#[async_trait]
impl MatrixClient for ReqwestMatrixClient {
    fn room_event_stream(
        &self,
        room_id: String,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> BoxStream<'static, Result<FederatedEvent, PortError>> {
        // Stub: returns an empty stream until Tuwunel dep is wired.
        // Replace with a long-poll /sync loop or a persistent WS connection.
        tracing::info!(
            room_id = %room_id,
            "MatrixBridge room_event_stream: stub — no events until Tuwunel is wired"
        );

        let _ = (room_id, tenant_id, channel_id);
        Box::pin(stream::empty())
    }

    async fn send_event(
        &self,
        room_id: &str,
        content: serde_json::Value,
    ) -> Result<(), MatrixBridgeError> {
        let txn_id = uuid::Uuid::new_v4();
        let url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
            self.homeserver_url, room_id, txn_id
        );

        self.http
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&content)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

/// Test-only in-memory Matrix client that yields a fixed set of events.
#[cfg(test)]
use crate::convert::matrix_event_to_federated;
#[cfg(test)]
pub struct MockMatrixClient {
    pub events: Vec<RawMatrixEvent>,
}

#[cfg(test)]
#[async_trait]
impl MatrixClient for MockMatrixClient {
    fn room_event_stream(
        &self,
        room_id: String,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> BoxStream<'static, Result<FederatedEvent, PortError>> {
        let projected: Vec<Result<FederatedEvent, PortError>> = self
            .events
            .iter()
            .cloned()
            .map(|raw| {
                matrix_event_to_federated(raw, &room_id, tenant_id, channel_id)
                    .map_err(PortError::from)
            })
            .collect();

        Box::pin(stream::iter(projected))
    }

    async fn send_event(
        &self,
        _room_id: &str,
        _content: serde_json::Value,
    ) -> Result<(), MatrixBridgeError> {
        Ok(())
    }
}
