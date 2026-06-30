#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use dashmap::DashMap;
use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::{MediaSignaler, PortError, SignalStream};
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;

/// Capacity of the per-session signal channel buffer.
const SIGNAL_CHANNEL_CAPACITY: usize = 64;

/// A str0m sans-I/O SFU session entry.
struct SfuSession {
    tx: mpsc::Sender<Result<SignalEnvelope, PortError>>,
    /// Reserved for future TTL-based eviction sweep.
    #[allow(dead_code)]
    created_at: Instant,
}

/// Sovereign SFU signaling adapter backed by `str0m`.
///
/// Each session gets a buffered channel. `send_signal` routes the envelope to
/// the correct session's sender; `subscribe_signals` returns a stream from the
/// receiver end of that channel. The str0m `Rtc` state machine is driven
/// externally via `process_offer` / `process_ice` calls when the gateway
/// receives SDP or ICE candidate signals from the gRPC `SignalService`.
pub struct StrOmSignaler {
    sessions: Arc<DashMap<(TenantId, SessionId), SfuSession>>,
}

impl StrOmSignaler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }
}

impl Default for StrOmSignaler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MediaSignaler for StrOmSignaler {
    #[instrument(name = "str0m::send_signal", skip(self, signal))]
    async fn send_signal(&self, signal: SignalEnvelope) -> Result<(), PortError> {
        let key = (signal.tenant_id, signal.from_session);
        let entry = self
            .sessions
            .get(&key)
            .ok_or_else(|| PortError::NotFound(format!("session {:?}", key.1)))?;

        entry
            .tx
            .send(Ok(signal))
            .await
            .map_err(|_| PortError::Transport("session channel closed".into()))
    }

    #[instrument(name = "str0m::subscribe_signals", skip(self))]
    async fn subscribe_signals(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        let (tx, rx) = mpsc::channel(SIGNAL_CHANNEL_CAPACITY);
        let session = SfuSession {
            tx,
            created_at: Instant::now(),
        };
        self.sessions.insert((tenant_id, session_id), session);

        tracing::debug!(%session_id, %tenant_id, "str0m: session registered");

        let stream = ReceiverStream::new(rx).map(|r| r);
        Ok(Box::pin(stream))
    }

    #[instrument(name = "str0m::remove_session", skip(self))]
    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError> {
        self.sessions.remove(&(tenant_id, session_id));
        tracing::debug!(%session_id, %tenant_id, "str0m: session removed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use frf_domain::{SessionId, SfuMode, SignalEnvelope, SignalKind, TenantId};
    use tokio_stream::StreamExt as _;

    fn make_envelope(tenant: TenantId, session: SessionId) -> SignalEnvelope {
        SignalEnvelope {
            tenant_id: tenant,
            from_session: session,
            to_session: None,
            room_id: "test-room".to_owned(),
            kind: SignalKind::Offer,
            sfu_mode: SfuMode::Sovereign,
            payload: serde_json::json!({"sdp": "v=0"}),
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn subscribe_then_send_delivers_signal() {
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let session = SessionId::new();

        let mut stream = signaler
            .subscribe_signals(session, tenant)
            .await
            .expect("subscribe");

        let envelope = make_envelope(tenant, session);
        signaler.send_signal(envelope.clone()).await.expect("send");

        let received = stream.next().await.expect("item").expect("ok");
        assert_eq!(received.from_session, session);
    }

    #[tokio::test]
    async fn send_to_missing_session_returns_not_found() {
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let session = SessionId::new();
        let envelope = make_envelope(tenant, session);

        let err = signaler
            .send_signal(envelope)
            .await
            .expect_err("should fail");
        assert!(matches!(err, PortError::NotFound(_)));
    }

    #[tokio::test]
    async fn remove_session_drops_channel() {
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let session = SessionId::new();

        signaler
            .subscribe_signals(session, tenant)
            .await
            .expect("subscribe");
        signaler
            .remove_session(session, tenant)
            .await
            .expect("remove");

        let envelope = make_envelope(tenant, session);
        let err = signaler
            .send_signal(envelope)
            .await
            .expect_err("should fail after removal");
        assert!(matches!(err, PortError::NotFound(_)));
    }
}
