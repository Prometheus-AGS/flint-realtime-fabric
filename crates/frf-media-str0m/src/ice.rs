//! Trickle-ICE envelope helpers for the str0m media transport.
//!
//! Candidates cross the `MediaTransport` port as [`SignalEnvelope`]s
//! (`SignalKind::IceCandidate`) — consistent with signaling. This module maps between an
//! envelope and a str0m candidate SDP string (inbound) and builds the outbound envelopes the
//! session broadcasts (its local host candidate + connection-state changes).
//!
//! Note (str0m sans-I/O model): there is no per-local-candidate str0m *event*. Local
//! candidates are advertised in the SDP answer (the host candidate seeded at bind) and
//! srflx candidates come from feeding STUN — the latter is browser/STUN-gated. So the
//! outbound stream carries the host candidate + state, not a live srflx trickle.

use chrono::Utc;
use frf_domain::{SessionId, SfuMode, SignalEnvelope, SignalKind, TenantId};
use frf_ports::ConnectionState;

/// Extract the candidate SDP string from an inbound `IceCandidate` envelope.
///
/// The candidate rides the envelope `payload` — either a bare JSON string, or an object
/// with a `"candidate"` field (the shape browsers send). Returns `None` if neither is present.
#[must_use]
pub fn candidate_string_from_envelope(envelope: &SignalEnvelope) -> Option<String> {
    match &envelope.payload {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => map
            .get("candidate")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        _ => None,
    }
}

/// Build an outbound `IceCandidate` envelope carrying `candidate` (an SDP candidate string)
/// for `session`, to relay the session's local candidate back over the signaling path.
#[must_use]
pub fn candidate_envelope(
    tenant_id: TenantId,
    session_id: SessionId,
    room_id: &str,
    candidate: &str,
) -> SignalEnvelope {
    SignalEnvelope {
        from_session: session_id,
        to_session: None,
        tenant_id,
        room_id: room_id.to_owned(),
        kind: SignalKind::IceCandidate,
        sfu_mode: SfuMode::Sovereign,
        payload: serde_json::json!({ "candidate": candidate }),
        timestamp: Utc::now(),
        subject: None,
    }
}

/// Build an outbound envelope announcing a session `state` change. Carried as an
/// `IceCandidate`-kind envelope with a `{"connectionState": …}` payload so it rides the same
/// signaling path; consumers key off the payload shape.
#[must_use]
pub fn state_envelope(
    tenant_id: TenantId,
    session_id: SessionId,
    room_id: &str,
    state: ConnectionState,
) -> SignalEnvelope {
    SignalEnvelope {
        from_session: session_id,
        to_session: None,
        tenant_id,
        room_id: room_id.to_owned(),
        kind: SignalKind::IceCandidate,
        sfu_mode: SfuMode::Sovereign,
        payload: serde_json::json!({ "connectionState": format!("{state:?}") }),
        timestamp: Utc::now(),
        subject: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_envelope(payload: serde_json::Value) -> SignalEnvelope {
        SignalEnvelope {
            from_session: SessionId::new(),
            to_session: None,
            tenant_id: TenantId::new(),
            room_id: "room-1".to_owned(),
            kind: SignalKind::IceCandidate,
            sfu_mode: SfuMode::Sovereign,
            payload,
            timestamp: Utc::now(),
            subject: None,
        }
    }

    #[test]
    fn extracts_candidate_from_bare_string_payload() {
        let env = base_envelope(serde_json::json!(
            "candidate:1 1 udp 2113 1.2.3.4 5000 typ host"
        ));
        assert_eq!(
            candidate_string_from_envelope(&env).as_deref(),
            Some("candidate:1 1 udp 2113 1.2.3.4 5000 typ host")
        );
    }

    #[test]
    fn extracts_candidate_from_object_payload() {
        let env = base_envelope(serde_json::json!({ "candidate": "candidate:xyz typ host" }));
        assert_eq!(
            candidate_string_from_envelope(&env).as_deref(),
            Some("candidate:xyz typ host")
        );
    }

    #[test]
    fn returns_none_for_payload_without_a_candidate() {
        let env = base_envelope(serde_json::json!({ "sdp": "v=0" }));
        assert!(candidate_string_from_envelope(&env).is_none());
    }

    #[test]
    fn candidate_envelope_round_trips_through_the_extractor() {
        let tid = TenantId::new();
        let sid = SessionId::new();
        let out = candidate_envelope(tid, sid, "room-9", "candidate:abc typ host");
        assert_eq!(out.kind, SignalKind::IceCandidate);
        assert_eq!(out.from_session, sid);
        // The outbound envelope is readable by the same inbound extractor.
        assert_eq!(
            candidate_string_from_envelope(&out).as_deref(),
            Some("candidate:abc typ host")
        );
    }

    #[test]
    fn state_envelope_carries_the_connection_state() {
        let env = state_envelope(
            TenantId::new(),
            SessionId::new(),
            "room-1",
            ConnectionState::Connected,
        );
        assert_eq!(
            env.payload.get("connectionState").and_then(|v| v.as_str()),
            Some("Connected")
        );
    }
}
