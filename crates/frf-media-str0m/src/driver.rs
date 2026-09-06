//! Per-session engine helpers for the str0m media plane.
//!
//! Extracted from `session.rs` (p21-c001). Since p29-c001 (ADR-008) the per-session driver
//! loop is gone — one shared-socket demux loop (`demux.rs`) owns every `Rtc`. This module keeps
//! the per-session command/meta types and the pure event/forward helpers that both the demux
//! loop and the tests use: `handle_event` (publish state + fan out inbound media) and
//! `apply_forwarded` (write forwarded media / keyframe requests into a session's `Rtc`).

use std::collections::HashMap;
use std::sync::Arc;

use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::ConnectionState;
use str0m::media::MediaKind;
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

/// Handle one str0m `Event`: publish connection-state changes (watch + outbound envelope),
/// record MID→kind mappings from `MediaAdded` (p36-c002h), and route inbound `MediaData` to
/// the room router for fan-out to co-room peers.
///
/// `mid_kinds` is this session's own MID→`MediaKind` map, built incrementally from
/// `Event::MediaAdded` events. It is passed mutably so newly negotiated m-lines are registered
/// immediately. The fan-out embeds `kind` in `ForwardedMedia` so the receiving demux loop can
/// look up the correct MID on the destination session (sender and receiver may assign different
/// MID numbers to the same media kind — p36-c002h).
pub(crate) fn handle_event(
    event: &Event,
    state_tx: &watch::Sender<ConnectionState>,
    local_signals_tx: &broadcast::Sender<SignalEnvelope>,
    router: &RoomRouter,
    meta: &SessionMeta,
    mid_kinds: &mut HashMap<str0m::media::Mid, MediaKind>,
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
    } else if let Event::MediaAdded(m) = event {
        // p36-c002h: record the MID→kind mapping for this session. Sender and receiver
        // independently assign MID numbers; without this map the receiving demux would use the
        // sender's MID to index into the receiver's Rtc, crossing audio/video tracks.
        mid_kinds.insert(m.mid, m.kind);
        tracing::debug!(session = %meta.session_id, mid = ?m.mid, kind = ?m.kind, "sovereign: media added");
    } else if let Event::MediaData(m) = event {
        // The first inbound MediaData proves RTP is arriving at this session — logged so a run
        // shows whether the sender's media reaches the SFU before fan-out.
        tracing::info!(session = %meta.session_id, room = %meta.room_id, mid = ?m.mid, kind = ?mid_kinds.get(&m.mid), bytes = m.data.len(), "sovereign: inbound MediaData → fan-out");
        // Fan this media out to the room's other members (ADR-006), sender→receiver.
        // `kind` and `params` are included so the receiver can map to its own MID (p36-c002h).
        let kind = mid_kinds.get(&m.mid).copied().unwrap_or(MediaKind::Video);
        router.forward(
            meta.session_id,
            &ForwardedMedia {
                kind,
                params: m.params,
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
///
/// `mid_kinds` is the **receiver's** MID→`MediaKind` map (built from its own `MediaAdded`
/// events). It is used by `write_forwarded` to look up the receiver's local MID for the
/// frame's codec kind, avoiding the audio/video track-crossing that occurs when the sender and
/// receiver assign different MID numbers to the same kind (p36-c002h).
pub(crate) fn apply_forwarded(
    rtc: &mut Rtc,
    frame: &ForwardedFrame,
    mid_kinds: &HashMap<str0m::media::Mid, MediaKind>,
) {
    match frame {
        ForwardedFrame::Media(media) => write_forwarded(rtc, media, mid_kinds),
        ForwardedFrame::KeyframeRequest(req) => {
            // Resolve THIS session's video MID rather than trusting `req.mid`. A keyframe
            // request is only meaningful for video, and the requester's MID numbering does not
            // match this sender's: in the p36 decode proof the browser negotiates MID 0=audio,
            // MID 1=video, so a request naming MID 0 lands on the sender's *audio* track,
            // `request_keyframe` rejects it as not applicable, and no keyframe is ever produced —
            // leaving a late-joining receiver stuck on P-frames with `framesDecoded=0`.
            //
            // This is the same MID-crossing class that `write_forwarded` fixes for media
            // (p36-c002h); the keyframe path needs the identical treatment. Fall back to the
            // requester's MID only when this session has no video track registered yet.
            let target_mid = keyframe_target_mid(mid_kinds, req.mid);

            let Some(mut writer) = rtc.writer(target_mid) else {
                tracing::debug!(mid = ?target_mid, "no writer for keyframe request — dropping");
                return;
            };
            // `request_keyframe` self-guards (errors if the kind/direction isn't possible);
            // that is an expected drop for a session that can't serve the request, so log it
            // at debug, not warn.
            if let Err(e) = writer.request_keyframe(req.rid, req.kind) {
                tracing::debug!(error = %e, mid = ?target_mid, "keyframe request not applicable — dropping");
            } else {
                // INFO, not debug: whether the proactive PLI actually reached a sender is the
                // decisive signal when diagnosing `framesDecoded=0`, and the p36 run's captured
                // log had no way to show it.
                tracing::info!(mid = ?target_mid, kind = ?req.kind, "sovereign: keyframe request applied → PLI to sender");
            }
        }
    }
}

/// Resolve which of *this* session's MIDs a forwarded keyframe request should target.
///
/// A keyframe (PLI/FIR) request is only meaningful for video, so this returns this session's
/// own video MID. `requested` — the MID named by the requester — is used only as a fallback
/// when this session has no video track registered yet, because sender and receiver number
/// their MIDs independently: honouring the requester's number lands the request on whatever
/// track happens to share that index here (in the p36 decode proof, `Mid("0")` was audio).
fn keyframe_target_mid(
    mid_kinds: &HashMap<str0m::media::Mid, MediaKind>,
    requested: str0m::media::Mid,
) -> str0m::media::Mid {
    mid_kinds
        .iter()
        .find(|&(_, &k)| k == MediaKind::Video)
        .map_or(requested, |(&mid, _)| mid)
}

/// Write a forwarded media frame to this session's `Rtc`.
///
/// p36-c002h: The sender's `media.mid` cannot be used directly as the receiver's writer key —
/// sender and receiver assign MID numbers independently. Instead, find the receiver's MID for
/// `media.kind` (audio / video) by scanning the receiver's own `mid_kinds` map. This ensures
/// video frames are always written to the receiver's video track and audio frames to audio,
/// regardless of MID numbering.
///
/// After resolving the receiver's MID, the sender's PT is translated to the receiver's local PT
/// via `Writer::match_params` so the browser sees a PT that matches its own SDP negotiation.
fn write_forwarded(
    rtc: &mut Rtc,
    media: &ForwardedMedia,
    mid_kinds: &HashMap<str0m::media::Mid, MediaKind>,
) {
    // Resolve the receiver's MID for this kind (audio/video). Fall back to the sender's MID
    // only if the receiver has no MediaAdded for this kind yet (race: receiver Rtc negotiated
    // but MediaAdded event not yet processed — extremely unlikely in practice).
    let receiver_mid = mid_kinds
        .iter()
        .find(|&(_, &k)| k == media.kind)
        .map_or(media.mid, |(&mid, _)| mid);

    let Some(writer) = rtc.writer(receiver_mid) else {
        tracing::debug!(mid = ?receiver_mid, kind = ?media.kind, "no writer for forwarded media — dropping");
        return;
    };
    // Translate the sender's PT to the receiver's local PT. The receiver may have negotiated
    // the same codec at a different PT number; `match_params` finds the match by codec name +
    // clock rate + format params. If no match is found the PT is used as-is.
    let pt = writer.match_params(media.params).unwrap_or(media.pt);
    if let Err(e) = writer.write(pt, media.network_time, media.time, media.data.clone()) {
        tracing::warn!(error = %e, kind = ?media.kind, pt = ?pt, "forwarded media write failed — dropping");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use str0m::media::Mid;

    /// The p36 decode-proof topology: the browser negotiates MID 0 = audio, MID 1 = video.
    fn browser_mid_kinds() -> HashMap<Mid, MediaKind> {
        HashMap::from([
            (Mid::from("0"), MediaKind::Audio),
            (Mid::from("1"), MediaKind::Video),
        ])
    }

    #[test]
    fn keyframe_request_targets_video_not_the_requesters_mid() {
        // Regression for the p36 `framesDecoded=0` stall: `join_room` sends its proactive PLI
        // with a placeholder `Mid("0")`, which on this session is AUDIO. Honouring that number
        // made `request_keyframe` reject the request as not applicable, so no keyframe was ever
        // emitted and a late-joining receiver stayed stuck on P-frames.
        let target = keyframe_target_mid(&browser_mid_kinds(), Mid::from("0"));
        assert_eq!(
            target,
            Mid::from("1"),
            "a keyframe request must target this session's video MID, not the requester's"
        );
    }

    #[test]
    fn keyframe_request_ignores_requester_mid_even_when_it_is_video_here() {
        // The requester's numbering carries no meaning for this session; resolution is always
        // by kind, so the answer does not depend on what the requester happened to name.
        let target = keyframe_target_mid(&browser_mid_kinds(), Mid::from("7"));
        assert_eq!(target, Mid::from("1"));
    }

    #[test]
    fn keyframe_request_falls_back_to_requested_mid_when_no_video_track_yet() {
        // Race: the session's `Rtc` is negotiated but its video `MediaAdded` has not been
        // processed. With nothing better to go on, keep the requester's MID.
        let audio_only = HashMap::from([(Mid::from("0"), MediaKind::Audio)]);
        let target = keyframe_target_mid(&audio_only, Mid::from("3"));
        assert_eq!(target, Mid::from("3"));
    }
}
