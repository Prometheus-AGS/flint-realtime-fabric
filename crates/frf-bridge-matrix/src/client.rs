use std::time::Duration;

use async_trait::async_trait;
use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{error::PortError, federation::FederatedEvent};
use futures_util::stream::BoxStream;

use crate::convert::matrix_event_to_federated;
use crate::error::MatrixBridgeError;

/// Long-poll timeout the homeserver holds a `/sync` open for (ms).
const SYNC_TIMEOUT_MS: u64 = 30_000;
/// Initial backoff after a failed `/sync`, doubled up to the cap.
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);
/// Maximum backoff between failed `/sync` attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(30);

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
/// Inbound: long-polls `/sync` (threading the `next_batch` token, with backoff) and
/// projects each room's timeline events. Outbound: HTTP PUT to the room send endpoint.
/// Both use bearer auth. A Tuwunel-native client could replace this later — but reqwest
/// `/sync` needs no extra dependency, so it is not blocked on the Tuwunel crate.
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
        // Real inbound: long-poll the Matrix Client-Server `/sync` endpoint, thread the
        // `next_batch` token forward, and project each new timeline event for this room
        // into a FederatedEvent. Reconnects with exponential backoff on transient errors.
        // (A Tuwunel-native client could replace this later, but reqwest /sync needs no
        // extra dependency.)
        let http = self.http.clone();
        let homeserver = self.homeserver_url.clone();
        let token = self.access_token.clone();

        Box::pin(async_stream::stream! {
            let mut since: Option<String> = None;
            let mut backoff = INITIAL_BACKOFF;

            loop {
                match sync_once(&http, &homeserver, &token, since.as_deref()).await {
                    Ok((next_batch, events)) => {
                        backoff = INITIAL_BACKOFF; // reset on success
                        since = Some(next_batch);
                        for raw in events_for_room(&events, &room_id) {
                            yield matrix_event_to_federated(raw, &room_id, tenant_id, channel_id)
                                .map_err(PortError::from);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(room_id = %room_id, error = %e, "matrix /sync failed — backing off");
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                    }
                }
            }
        })
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

/// Perform one `GET /sync` long-poll. Returns the `next_batch` token and the parsed
/// response body. `since` threads the previous batch forward (absent on the first poll).
async fn sync_once(
    http: &reqwest::Client,
    homeserver: &str,
    token: &str,
    since: Option<&str>,
) -> Result<(String, serde_json::Value), MatrixBridgeError> {
    let url = format!("{homeserver}/_matrix/client/v3/sync");
    let timeout = SYNC_TIMEOUT_MS.to_string();
    let mut query: Vec<(&str, &str)> = vec![("timeout", &timeout)];
    if let Some(s) = since {
        query.push(("since", s));
    }

    let body: serde_json::Value = http
        .get(&url)
        .bearer_auth(token)
        .query(&query)
        // Give the request a little longer than the server-side long-poll timeout.
        .timeout(Duration::from_millis(SYNC_TIMEOUT_MS + 10_000))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let next_batch = body
        .get("next_batch")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();

    Ok((next_batch, body))
}

/// Extract this room's new timeline events from a `/sync` response body:
/// `rooms.join.<room_id>.timeline.events[]`.
fn events_for_room(body: &serde_json::Value, room_id: &str) -> Vec<RawMatrixEvent> {
    let Some(events) = body
        .get("rooms")
        .and_then(|r| r.get("join"))
        .and_then(|j| j.get(room_id))
        .and_then(|room| room.get("timeline"))
        .and_then(|t| t.get("events"))
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };

    events
        .iter()
        .map(|ev| RawMatrixEvent {
            event_id: ev
                .get("event_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            sender: ev
                .get("sender")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            content: ev
                .get("content")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        })
        .collect()
}

/// Test-only in-memory Matrix client that yields a fixed set of events.
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

        Box::pin(futures_util::stream::iter(projected))
    }

    async fn send_event(
        &self,
        _room_id: &str,
        _content: serde_json::Value,
    ) -> Result<(), MatrixBridgeError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_for_room_extracts_this_rooms_timeline() {
        // A realistic /sync body: two events in the target room, one in another room.
        let body = serde_json::json!({
            "next_batch": "s72_1",
            "rooms": {
                "join": {
                    "!target:hs": {
                        "timeline": {
                            "events": [
                                { "event_id": "$e1", "sender": "@a:hs", "content": { "body": "hi" } },
                                { "event_id": "$e2", "sender": "@b:hs", "content": { "body": "yo" } }
                            ]
                        }
                    },
                    "!other:hs": {
                        "timeline": { "events": [{ "event_id": "$x", "content": {} }] }
                    }
                }
            }
        });

        let events = events_for_room(&body, "!target:hs");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_id.as_deref(), Some("$e1"));
        assert_eq!(events[0].sender.as_deref(), Some("@a:hs"));
        assert_eq!(events[1].event_id.as_deref(), Some("$e2"));
    }

    #[test]
    fn events_for_room_returns_empty_when_room_absent() {
        let body = serde_json::json!({ "next_batch": "s1", "rooms": { "join": {} } });
        assert!(events_for_room(&body, "!missing:hs").is_empty());
    }

    #[test]
    fn events_for_room_handles_malformed_body() {
        // No rooms key at all → empty, not a panic.
        let body = serde_json::json!({ "next_batch": "s1" });
        assert!(events_for_room(&body, "!any:hs").is_empty());
    }
}
