//! Room registry + RTP fan-out router for the sovereign SFU (ADR-006).
//!
//! Each session's `Rtc` runs in its own isolated driver task ([`crate::driver`]), so
//! forwarding media A→B is cross-task message passing. This module centralizes that: a
//! session's driver, on `Event::MediaData`, calls [`RoomRouter::forward`], which pushes a
//! [`ForwardedMedia`] to every *other* room member's per-session forwarding channel; their
//! drivers then write it to their `Rtc` via `writer(mid).write`.
//!
//! Phase-21 proves **1-to-1** forwarding; N-peer fan-out + PLI are phase-22. The forwarding
//! channel is bounded — a full channel drops the packet (RTP is loss-tolerant) rather than
//! blocking the source driver.

use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use frf_domain::{SessionId, TenantId};
use str0m::format::PayloadParams;
use str0m::media::{KeyframeRequest, MediaKind, MediaTime, Mid, Pt};
use tokio::sync::mpsc;

/// One media frame handed from a source session's driver to a destination's driver.
///
/// `data` is `Arc<[u8]>` (str0m's own payload type) so the cross-task hand-off is a cheap
/// refcount bump, not a copy.
///
/// `kind` (audio / video) and `params` are carried from the sender's `MediaData` so the
/// receiver can locate its own MID for this kind (p36-c002h: sender and receiver may assign
/// different MID numbers to the same media kind) and translate the PT via `match_params`.
#[derive(Debug, Clone)]
pub struct ForwardedMedia {
    pub kind: MediaKind,
    pub params: PayloadParams,
    pub mid: Mid,
    pub pt: Pt,
    pub time: MediaTime,
    pub network_time: std::time::Instant,
    pub data: Arc<[u8]>,
}

/// A frame handed across the per-session forwarding channel: either media (sender→receiver)
/// or a keyframe request (receiver→sender). One channel carries both directions of fan-out.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ForwardedFrame {
    /// Media to write to the destination's `Rtc`.
    Media(ForwardedMedia),
    /// A keyframe (PLI/FIR) request to apply to the destination's `Rtc`.
    KeyframeRequest(KeyframeRequest),
}

/// Central room membership + per-session forwarding channels. Shared (`Arc`) across all
/// session driver tasks.
#[derive(Default)]
pub struct RoomRouter {
    /// Room → the sessions currently in it.
    rooms: DashMap<(TenantId, String), HashSet<SessionId>>,
    /// Session → its driver's forwarding channel sender.
    forwarders: DashMap<SessionId, mpsc::Sender<ForwardedFrame>>,
    /// Session → the room it belongs to (for deregistration + forward lookup).
    membership: DashMap<SessionId, (TenantId, String)>,
}

impl RoomRouter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a session in a room with its forwarding-channel sender.
    pub fn register(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
        room: &str,
        forward_tx: mpsc::Sender<ForwardedFrame>,
    ) {
        let key = (tenant_id, room.to_owned());
        self.rooms
            .entry(key.clone())
            .or_default()
            .insert(session_id);
        self.forwarders.insert(session_id, forward_tx);
        self.membership.insert(session_id, key);
    }

    /// Remove a session from its room and drop its forwarding channel.
    pub fn deregister(&self, session_id: SessionId) {
        self.forwarders.remove(&session_id);
        if let Some((_, key)) = self.membership.remove(&session_id)
            && let Some(mut members) = self.rooms.get_mut(&key)
        {
            members.remove(&session_id);
        }
    }

    /// The number of *other* members `from` would forward to in its room. Test-only —
    /// lets the `StrOmTransport` wiring (create/join/remove) be asserted without exposing a
    /// public introspection surface.
    #[cfg(test)]
    pub(crate) fn peer_count(&self, from: SessionId) -> usize {
        let Some(key) = self.membership.get(&from).map(|k| k.clone()) else {
            return 0;
        };
        self.rooms
            .get(&key)
            .map_or(0, |m| m.iter().filter(|&&s| s != from).count())
    }

    /// Forward `media` from `from` to every *other* member of its room (sender→receiver).
    pub fn forward(&self, from: SessionId, media: &ForwardedMedia) {
        self.fan_out(from, &ForwardedFrame::Media(media.clone()));
    }

    /// Forward a keyframe (PLI/FIR) `request` from `from` to every *other* member of its room
    /// (receiver→sender — the reverse direction to media).
    pub fn forward_keyframe_request(&self, from: SessionId, request: KeyframeRequest) {
        self.fan_out(from, &ForwardedFrame::KeyframeRequest(request));
    }

    /// Push `frame` to every *other* member of `from`'s room. A destination whose forwarding
    /// channel is full has the frame dropped (RTP is loss-tolerant), never blocking the caller.
    fn fan_out(&self, from: SessionId, frame: &ForwardedFrame) {
        let Some(key) = self.membership.get(&from).map(|k| k.clone()) else {
            return;
        };
        let Some(members) = self.rooms.get(&key) else {
            return;
        };
        for peer in members.iter().filter(|&&s| s != from) {
            if let Some(tx) = self.forwarders.get(peer) {
                // Full → drop (loss-tolerant); Closed → driver ended (deregisters on remove).
                // Both are non-fatal no-ops; only a full channel is worth a warning.
                if let Err(mpsc::error::TrySendError::Full(_)) = tx.try_send(frame.clone()) {
                    tracing::warn!(?peer, "forwarding channel full — dropping frame");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media() -> ForwardedMedia {
        use str0m::format::{Codec, CodecSpec, FormatParams, PayloadParams};
        use str0m::media::Frequency;
        let spec = CodecSpec {
            codec: Codec::Vp8,
            clock_rate: Frequency::NINETY_KHZ,
            channels: None,
            format: FormatParams::default(),
        };
        ForwardedMedia {
            kind: MediaKind::Video,
            params: PayloadParams::new(Pt::new_with_value(96), None, spec),
            mid: Mid::from("0"),
            pt: Pt::new_with_value(96),
            time: MediaTime::ZERO,
            network_time: std::time::Instant::now(),
            data: Arc::from([1_u8, 2, 3].as_slice()),
        }
    }

    #[tokio::test]
    async fn forward_delivers_to_the_other_room_member_not_the_sender() {
        let router = RoomRouter::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();
        let (a_tx, mut a_rx) = mpsc::channel(4);
        let (b_tx, mut b_rx) = mpsc::channel(4);
        router.register(a, tenant, "room-1", a_tx);
        router.register(b, tenant, "room-1", b_tx);

        router.forward(a, &media());

        // B receives it; A (the sender) does not.
        assert!(b_rx.try_recv().is_ok(), "peer B should receive the frame");
        assert!(
            a_rx.try_recv().is_err(),
            "sender A should not receive its own frame"
        );
    }

    #[tokio::test]
    async fn deregister_removes_a_session_from_forwarding() {
        let router = RoomRouter::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();
        let (a_tx, _a_rx) = mpsc::channel(4);
        let (b_tx, mut b_rx) = mpsc::channel(4);
        router.register(a, tenant, "room-1", a_tx);
        router.register(b, tenant, "room-1", b_tx);

        router.deregister(b);
        router.forward(a, &media());
        assert!(
            b_rx.try_recv().is_err(),
            "a deregistered peer receives nothing"
        );
    }

    #[tokio::test]
    async fn forward_fans_out_to_all_other_room_members() {
        // N-peer: three sessions in one room. A sends → B and C each receive; A does not.
        // Proves `forward` fans to *all* co-room peers (phase-21 only proved the 2-peer case).
        let router = RoomRouter::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();
        let c = SessionId::new();
        let (a_tx, mut a_rx) = mpsc::channel(4);
        let (b_tx, mut b_rx) = mpsc::channel(4);
        let (c_tx, mut c_rx) = mpsc::channel(4);
        router.register(a, tenant, "room-1", a_tx);
        router.register(b, tenant, "room-1", b_tx);
        router.register(c, tenant, "room-1", c_tx);

        router.forward(a, &media());

        assert!(b_rx.try_recv().is_ok(), "peer B receives the frame");
        assert!(c_rx.try_recv().is_ok(), "peer C receives the frame");
        assert!(
            a_rx.try_recv().is_err(),
            "sender A does not receive its own frame"
        );
    }

    #[tokio::test]
    async fn keyframe_request_routes_to_the_other_member_not_the_requester() {
        // A keyframe request travels receiver→sender: B (a receiver) requests, A (the sender)
        // gets it as a `ForwardedFrame::KeyframeRequest`; B does not get its own request back.
        let router = RoomRouter::new();
        let tenant = TenantId::new();
        let a = SessionId::new();
        let b = SessionId::new();
        let (a_tx, mut a_rx) = mpsc::channel(4);
        let (b_tx, mut b_rx) = mpsc::channel(4);
        router.register(a, tenant, "room-1", a_tx);
        router.register(b, tenant, "room-1", b_tx);

        let req = KeyframeRequest {
            mid: Mid::from("0"),
            rid: None,
            kind: str0m::media::KeyframeRequestKind::Pli,
        };
        router.forward_keyframe_request(b, req);

        match a_rx.try_recv() {
            Ok(ForwardedFrame::KeyframeRequest(_)) => {}
            other => panic!("A should receive a keyframe request, got {other:?}"),
        }
        assert!(
            b_rx.try_recv().is_err(),
            "the requester B does not receive its own request"
        );
    }
}
