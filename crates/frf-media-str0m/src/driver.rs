//! Per-session engine helpers for the str0m media plane.
//!
//! Extracted from `session.rs` (p21-c001). Since p29-c001 (ADR-008) the per-session driver
//! loop is gone — one shared-socket demux loop (`demux.rs`) owns every `Rtc`. This module keeps
//! the per-session command/meta types and the pure event/forward helpers that both the demux
//! loop and the tests use: `handle_event` (publish state + fan out inbound media) and
//! `apply_forwarded` (write forwarded media / keyframe requests into a session's `Rtc`).

use std::sync::Arc;

use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::ConnectionState;
use str0m::{Event, IceConnectionState, Rtc};
use tokio::sync::{broadcast, watch};

use crate::room::{ForwardedFrame, ForwardedMedia, RoomRouter};

/// Max inbound UDP datagram size we buffer.
pub(crate) const RECV_BUF: usize = 2000;

/// Commands sent from the `MediaTransport` surface to a session's driver task.
#[derive(Debug)]
pub(crate) enum SessionCommand {
    /// Feed a remote trickle-ICE candidate string into the session's `Rtc`.
    AddRemoteCandidate(String),
}

/// Identity a session's driver needs to stamp its outbound `SignalEnvelope`s.
// The `_id`/`room_id` suffixes mirror the domain field names (`SignalEnvelope.session_id`,
// `.tenant_id`, `.room_id`) — keeping them aligned is clearer than renaming to satisfy the
// shared-suffix lint.
#[allow(clippy::struct_field_names)]
#[derive(Clone)]
pub(crate) struct SessionMeta {
    pub(crate) session_id: SessionId,
    pub(crate) tenant_id: TenantId,
    pub(crate) room_id: String,
}

/// Map a str0m connection event to our port's [`ConnectionState`], if it represents a state
/// change worth publishing.
fn state_for_event(event: &Event) -> Option<ConnectionState> {
    match event {
        Event::Connected => Some(ConnectionState::Connected),
        Event::IceConnectionStateChange(ice) => Some(match ice {
            IceConnectionState::New | IceConnectionState::Checking => ConnectionState::Connecting,
            IceConnectionState::Connected | IceConnectionState::Completed => {
                ConnectionState::Connected
            }
            IceConnectionState::Disconnected => ConnectionState::Disconnected,
        }),
        _ => None,
    }
}

/// Handle one str0m `Event`: publish connection-state changes (watch + outbound envelope)
/// and route inbound `MediaData` to the room router for fan-out to co-room peers.
pub(crate) fn handle_event(
    event: &Event,
    state_tx: &watch::Sender<ConnectionState>,
    local_signals_tx: &broadcast::Sender<SignalEnvelope>,
    router: &RoomRouter,
    meta: &SessionMeta,
) {
    if let Some(state) = state_for_event(event) {
        // Lifecycle visibility (p27-c001): a live decode run shows whether the session reaches
        // Connected and when — the key signal for diagnosing a stalled ICE/DTLS negotiation.
        tracing::info!(session = %meta.session_id, room = %meta.room_id, ?state, "sovereign: connection state");
        // Ignore send errors — a dropped receiver means the session is gone.
        let _ = state_tx.send(state);
        let _ = local_signals_tx.send(crate::ice::state_envelope(
            meta.tenant_id,
            meta.session_id,
            &meta.room_id,
            state,
        ));
    } else if let Event::MediaData(m) = event {
        // The first inbound MediaData proves RTP is arriving at this session — logged so a run
        // shows whether the sender's media reaches the SFU before fan-out.
        tracing::info!(session = %meta.session_id, room = %meta.room_id, mid = ?m.mid, bytes = m.data.len(), "sovereign: inbound MediaData → fan-out");
        // Fan this media out to the room's other members (ADR-006), sender→receiver.
        router.forward(
            meta.session_id,
            &ForwardedMedia {
                mid: m.mid,
                pt: m.pt,
                time: m.time,
                network_time: m.network_time,
                data: Arc::clone(&m.data),
            },
        );
    } else if let Event::KeyframeRequest(req) = event {
        // A keyframe request travels receiver→sender: relay it to the room's other members.
        router.forward_keyframe_request(meta.session_id, *req);
    }
}

/// Apply a forwarded frame to this session's `Rtc`: write media, or apply a keyframe request.
pub(crate) fn apply_forwarded(rtc: &mut Rtc, frame: &ForwardedFrame) {
    match frame {
        ForwardedFrame::Media(media) => write_forwarded(rtc, media),
        ForwardedFrame::KeyframeRequest(req) => {
            let Some(mut writer) = rtc.writer(req.mid) else {
                tracing::debug!(mid = ?req.mid, "no writer for keyframe request — dropping");
                return;
            };
            // `request_keyframe` self-guards (errors if the kind/direction isn't possible);
            // that is an expected drop for a session that can't serve the request, so log it
            // at debug, not warn.
            if let Err(e) = writer.request_keyframe(req.rid, req.kind) {
                tracing::debug!(error = %e, "keyframe request not applicable — dropping");
            }
        }
    }
}

/// Write a forwarded media frame to this session's `Rtc`. A missing writer (mid not yet
/// negotiated) or an `UnknownPt` (the peer negotiated a different codec — the ADR-006 caveat)
/// is logged and dropped, never fatal.
fn write_forwarded(rtc: &mut Rtc, media: &ForwardedMedia) {
    let Some(writer) = rtc.writer(media.mid) else {
        tracing::debug!(mid = ?media.mid, "no writer for forwarded media — dropping");
        return;
    };
    if let Err(e) = writer.write(media.pt, media.network_time, media.time, media.data.clone()) {
        tracing::warn!(error = %e, "forwarded media write failed — dropping");
    }
}
