//! Cross-node inbound relay for the `LiveKit` adapter.
//!
//! Outbound signals already fan out across nodes via `LiveKit`'s `send_data` (see
//! [`crate::adapter`]). The missing half was **inbound**: a signal published on another
//! gateway node reaches `LiveKit` room participants but was never re-surfaced through this
//! adapter locally. This module closes that gap behind a trait seam so the forwarding logic
//! is unit-testable without a live server or `libwebrtc`.
//!
//! A [`LiveKitDataSource`] yields the next server-originated data-channel payload;
//! [`spawn_inbound_relay`] deserializes each payload into a [`SignalEnvelope`] and forwards
//! it through the adapter's per-session fan-out — the exact topology the outbound path uses.
//! The real `libwebrtc`-backed data source is gated behind the off-by-default `realtime`
//! cargo feature so the default gateway build stays light.

use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::SignalEnvelope;

use crate::adapter::LiveKitSignaling;

/// Source of server-originated `LiveKit` data-channel payloads (one room's inbound stream).
///
/// Implementors return the next raw payload (the JSON-encoded [`SignalEnvelope`] bytes
/// that a peer node published via `send_data`), or `None` when the stream ends. The trait
/// seam keeps the relay loop testable with an in-memory source; the `libwebrtc`-backed
/// implementation is feature-gated (`realtime`).
#[async_trait]
pub trait LiveKitDataSource: Send + Sync {
    /// Await the next inbound payload, or `None` when the source is exhausted/closed.
    async fn next_payload(&self) -> Option<Vec<u8>>;
}

impl LiveKitSignaling {
    /// Spawn the inbound relay loop for a data source, returning its task handle.
    ///
    /// The loop reads payloads, deserializes each into a [`SignalEnvelope`], and forwards
    /// it to subscribed session streams. Malformed payloads are skipped with a warning —
    /// a bad frame from the network never takes the loop down.
    #[must_use]
    pub fn start_inbound_relay<S>(self: &Arc<Self>, source: Arc<S>) -> tokio::task::JoinHandle<()>
    where
        S: LiveKitDataSource + 'static,
    {
        let adapter = Arc::clone(self);
        tokio::spawn(async move { spawn_inbound_relay(adapter, source).await })
    }
}

/// Drive one data source to exhaustion, forwarding every valid signal into the adapter.
///
/// Factored out of [`LiveKitSignaling::start_inbound_relay`] so it can be awaited directly
/// in tests without spawning a task.
pub async fn spawn_inbound_relay<S>(adapter: Arc<LiveKitSignaling>, source: Arc<S>)
where
    S: LiveKitDataSource + ?Sized,
{
    while let Some(payload) = source.next_payload().await {
        match serde_json::from_slice::<SignalEnvelope>(&payload) {
            Ok(envelope) => adapter.fan_out(&envelope),
            Err(e) => {
                tracing::warn!(error = %e, "livekit inbound relay: skipping malformed payload");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_domain::{SessionId, SignalEnvelope, TenantId};
    use frf_ports::MediaSignaler as _;
    use tokio::sync::Mutex;
    use tokio_stream::StreamExt as _;
    use uuid::Uuid;

    use crate::config::LiveKitConfig;

    fn test_config() -> LiveKitConfig {
        LiveKitConfig {
            api_key: "key".into(),
            api_secret: "secret".into(),
            server_url: "https://example.livekit.cloud".into(),
            room_prefix: "frf/".into(),
        }
    }

    /// In-memory data source that yields a fixed queue of payloads, then closes.
    struct VecSource {
        payloads: Mutex<std::collections::VecDeque<Vec<u8>>>,
    }

    impl VecSource {
        fn new(payloads: Vec<Vec<u8>>) -> Self {
            Self {
                payloads: Mutex::new(payloads.into()),
            }
        }
    }

    #[async_trait]
    impl LiveKitDataSource for VecSource {
        async fn next_payload(&self) -> Option<Vec<u8>> {
            self.payloads.lock().await.pop_front()
        }
    }

    /// Build the JSON-encoded wire payload for a signal in `room`, exactly as a peer node
    /// would publish it via `send_data`. Built from a JSON literal (round-tripped through
    /// `SignalEnvelope`) so the test does not depend on the struct's exact field types.
    fn sample_payload(room: &str) -> Vec<u8> {
        let value = serde_json::json!({
            "from_session": SessionId::new().as_uuid(),
            "to_session": null,
            "tenant_id": Uuid::nil(),
            "room_id": room,
            "kind": "offer",
            "sfu_mode": "hosted",
            "payload": { "sdp": "v=0" },
            "timestamp": "2026-07-07T00:00:00Z",
        });
        // Validate it deserializes into a real SignalEnvelope before using it as input.
        let envelope: SignalEnvelope =
            serde_json::from_value(value).expect("sample must be a valid SignalEnvelope");
        serde_json::to_vec(&envelope).expect("serialize")
    }

    #[tokio::test]
    async fn valid_inbound_payload_is_forwarded_to_a_subscribed_session() {
        let adapter = Arc::new(LiveKitSignaling::new(test_config()));
        let session_id = SessionId::new();
        let tenant_id = TenantId::from_uuid(Uuid::nil());

        let mut stream = adapter
            .subscribe_signals(session_id, tenant_id)
            .await
            .expect("subscribe should succeed");

        let source = Arc::new(VecSource::new(vec![sample_payload("room-x")]));

        // Await the relay directly (no spawn) so the test is deterministic.
        spawn_inbound_relay(Arc::clone(&adapter), source).await;

        let received = stream
            .next()
            .await
            .expect("a signal")
            .expect("no stream err");
        assert_eq!(received.room_id, "room-x");
    }

    #[tokio::test]
    async fn malformed_inbound_payload_is_skipped_without_panic() {
        let adapter = Arc::new(LiveKitSignaling::new(test_config()));
        let session_id = SessionId::new();
        let tenant_id = TenantId::from_uuid(Uuid::nil());

        let mut stream = adapter
            .subscribe_signals(session_id, tenant_id)
            .await
            .expect("subscribe should succeed");

        // First payload is garbage, second is a valid signal — the loop must skip the
        // garbage and still deliver the valid one.
        let source = Arc::new(VecSource::new(vec![
            b"not json".to_vec(),
            sample_payload("room-y"),
        ]));

        spawn_inbound_relay(Arc::clone(&adapter), source).await;

        let received = stream
            .next()
            .await
            .expect("a signal")
            .expect("no stream err");
        assert_eq!(received.room_id, "room-y");
    }
}
