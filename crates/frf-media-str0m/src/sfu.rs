#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use dashmap::DashMap;
use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::{MediaSignaler, PortError, SignalStream};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;

/// Capacity of the per-session signal channel buffer.
const SIGNAL_CHANNEL_CAPACITY: usize = 64;

/// A str0m sans-I/O SFU session entry.
struct SfuSession {
    tx: mpsc::Sender<Result<SignalEnvelope, PortError>>,
    /// Room this session joined, for fan-out and cleanup on removal.
    room_id: String,
    /// Reserved for future TTL-based eviction sweep.
    #[allow(dead_code)]
    created_at: Instant,
}

/// Sovereign SFU **signaling** adapter (str0m media plane deferred).
///
/// Each session gets a buffered channel. `send_signal` delivers to the **target** peer —
/// unicast to `to_session` when set, otherwise fan-out to every other member of
/// `room_id` — so SDP/ICE actually reach the other peer(s). `subscribe_signals` returns
/// the receiver end. This moves signaling envelopes between peers but does **not** yet
/// drive a real WebRTC media plane — there is no `str0m::Rtc` state machine, SDP/ICE
/// negotiation, or RTP forwarding here. Sovereign mode is therefore gated off by default
/// (`SFU_MODE=hosted`) and boots with a warning; wiring real str0m media is a deferred
/// follow-up.
pub struct StrOmSignaler {
    sessions: Arc<DashMap<(TenantId, SessionId), SfuSession>>,
    /// Room membership index for broadcast fan-out: `(tenant, room_id) → session ids`.
    rooms: Arc<DashMap<(TenantId, String), HashSet<SessionId>>>,
}

impl StrOmSignaler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            rooms: Arc::new(DashMap::new()),
        }
    }

    /// Deliver an envelope to a single session's channel. Returns `NotFound` if the
    /// target session is not registered.
    async fn deliver_to(
        &self,
        key: &(TenantId, SessionId),
        signal: SignalEnvelope,
    ) -> Result<(), PortError> {
        let tx = {
            let entry = self
                .sessions
                .get(key)
                .ok_or_else(|| PortError::NotFound(format!("session {:?}", key.1)))?;
            entry.tx.clone()
        };
        tx.send(Ok(signal))
            .await
            .map_err(|_| PortError::Transport("session channel closed".into()))
    }

    /// Record that a registered session belongs to a room, so broadcast fan-out can
    /// reach it. A session learns its room from the first signal it sends (the envelope
    /// carries `room_id`). Idempotent; a no-op if the session isn't registered.
    fn join_room(&self, tenant_id: TenantId, session_id: SessionId, room_id: &str) {
        if room_id.is_empty() {
            return;
        }
        if let Some(mut session) = self.sessions.get_mut(&(tenant_id, session_id)) {
            if session.room_id == room_id {
                return; // already joined
            }
            // Move the session out of any previous room before joining the new one.
            if !session.room_id.is_empty() {
                if let Some(mut prev) = self.rooms.get_mut(&(tenant_id, session.room_id.clone())) {
                    prev.remove(&session_id);
                }
            }
            room_id.clone_into(&mut session.room_id);
        } else {
            return; // not a registered session — nothing to join
        }
        self.rooms
            .entry((tenant_id, room_id.to_owned()))
            .or_default()
            .insert(session_id);
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
        let tenant = signal.tenant_id;

        // The sender declares its room via the envelope; record membership so future
        // broadcasts from other peers reach it and so this broadcast can enumerate peers.
        self.join_room(tenant, signal.from_session, &signal.room_id);

        // Unicast: deliver to the explicit target peer.
        if let Some(target) = signal.to_session {
            return self.deliver_to(&(tenant, target), signal).await;
        }

        // Broadcast: fan out to every OTHER member of the room. Snapshot the member set
        // first so we don't hold the DashMap ref across awaits.
        let members: Vec<SessionId> = {
            let Some(room) = self.rooms.get(&(tenant, signal.room_id.clone())) else {
                return Err(PortError::NotFound(format!("room {}", signal.room_id)));
            };
            room.iter()
                .copied()
                .filter(|s| *s != signal.from_session)
                .collect()
        };

        // A room with only the sender is not an error — there is simply no one to notify.
        for target in members {
            // Ignore a single closed/absent receiver so one dead peer can't fail the
            // whole broadcast; the session map is the source of truth for liveness.
            let _ = self.deliver_to(&(tenant, target), signal.clone()).await;
        }
        Ok(())
    }

    #[instrument(name = "str0m::subscribe_signals", skip(self))]
    async fn subscribe_signals(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        let (tx, rx) = mpsc::channel(SIGNAL_CHANNEL_CAPACITY);
        // Room is learned from the first signal the session sends; until then it belongs
        // to no room. subscribe registers the channel; join_room updates membership.
        let session = SfuSession {
            tx,
            room_id: String::new(),
            created_at: Instant::now(),
        };
        self.sessions.insert((tenant_id, session_id), session);

        tracing::debug!(%session_id, %tenant_id, "str0m: session registered");

        let stream = ReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }

    #[instrument(name = "str0m::remove_session", skip(self))]
    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError> {
        // Drop the channel and remove the session from its room's membership set.
        if let Some((_, session)) = self.sessions.remove(&(tenant_id, session_id)) {
            if !session.room_id.is_empty() {
                if let Some(mut room) = self.rooms.get_mut(&(tenant_id, session.room_id.clone())) {
                    room.remove(&session_id);
                }
            }
        }
        tracing::debug!(%session_id, %tenant_id, "str0m: session removed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use frf_domain::{SessionId, SfuMode, SignalEnvelope, SignalKind, TenantId};
    use futures_util::FutureExt as _; // now_or_never
    use tokio_stream::StreamExt as _;

    /// Build a signal from `from` to `to` (unicast when `to` is `Some`, broadcast when
    /// `None`) within `room`.
    fn envelope(
        tenant: TenantId,
        from: SessionId,
        to: Option<SessionId>,
        room: &str,
    ) -> SignalEnvelope {
        SignalEnvelope {
            tenant_id: tenant,
            from_session: from,
            to_session: to,
            room_id: room.to_owned(),
            kind: SignalKind::Offer,
            sfu_mode: SfuMode::Sovereign,
            payload: serde_json::json!({"sdp": "v=0"}),
            timestamp: Utc::now(),
            subject: None,
        }
    }

    #[tokio::test]
    async fn unicast_delivers_to_target_peer_not_sender() {
        // The bug this fixes: a signal from A must reach B (its to_session), NOT be
        // echoed back to A. Subscribe as both A and B; A sends to B; only B receives.
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();

        let mut a_stream = signaler.subscribe_signals(a, tenant).await.expect("sub a");
        let mut b_stream = signaler.subscribe_signals(b, tenant).await.expect("sub b");

        signaler
            .send_signal(envelope(tenant, a, Some(b), "room-1"))
            .await
            .expect("send a→b");

        // B receives it, tagged as from A.
        let received = b_stream.next().await.expect("b item").expect("ok");
        assert_eq!(received.from_session, a);
        assert_eq!(received.to_session, Some(b));

        // A must NOT receive its own signal — the old bug delivered it back to the sender.
        assert!(
            a_stream.next().now_or_never().is_none(),
            "sender must not receive its own signal"
        );
    }

    #[tokio::test]
    async fn broadcast_reaches_all_other_room_members() {
        // to_session = None → fan out to every OTHER member of the room.
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();
        let c = SessionId::new();

        let mut a_stream = signaler.subscribe_signals(a, tenant).await.expect("sub a");
        let mut b_stream = signaler.subscribe_signals(b, tenant).await.expect("sub b");
        let mut c_stream = signaler.subscribe_signals(c, tenant).await.expect("sub c");

        // B and C join the room by sending into it first (room is learned from signals).
        signaler
            .send_signal(envelope(tenant, b, Some(a), "room-1"))
            .await
            .expect("b joins");
        signaler
            .send_signal(envelope(tenant, c, Some(a), "room-1"))
            .await
            .expect("c joins");
        // Drain A's two unicast joins so the assertions below only see the broadcast.
        let _ = a_stream.next().await;
        let _ = a_stream.next().await;

        // A broadcasts to the room → B and C get it, A does not.
        signaler
            .send_signal(envelope(tenant, a, None, "room-1"))
            .await
            .expect("a broadcast");

        let to_b = b_stream.next().await.expect("b item").expect("ok");
        let to_c = c_stream.next().await.expect("c item").expect("ok");
        assert_eq!(to_b.from_session, a);
        assert_eq!(to_c.from_session, a);
        assert!(
            a_stream.next().now_or_never().is_none(),
            "broadcaster must not receive its own broadcast"
        );
    }

    #[tokio::test]
    async fn unicast_to_missing_session_returns_not_found() {
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let missing = SessionId::new();

        let err = signaler
            .send_signal(envelope(tenant, SessionId::new(), Some(missing), "room-1"))
            .await
            .expect_err("should fail");
        assert!(matches!(err, PortError::NotFound(_)));
    }

    #[tokio::test]
    async fn remove_session_deregisters_and_leaves_room() {
        let signaler = StrOmSignaler::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();

        let _a = signaler.subscribe_signals(a, tenant).await.expect("sub a");
        let _b = signaler.subscribe_signals(b, tenant).await.expect("sub b");
        // A joins the room, then B unicasts to A so A is a room member and reachable.
        signaler
            .send_signal(envelope(tenant, a, Some(b), "room-1"))
            .await
            .expect("a joins");

        signaler.remove_session(a, tenant).await.expect("remove a");

        // Unicast to the removed session now fails.
        let err = signaler
            .send_signal(envelope(tenant, b, Some(a), "room-1"))
            .await
            .expect_err("should fail after removal");
        assert!(matches!(err, PortError::NotFound(_)));
    }
}
