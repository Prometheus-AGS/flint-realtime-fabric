use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{SessionId, TenantId};

/// WebRTC signaling message type.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Offer,
    Answer,
    IceCandidate,
    IceRestart,
    Hangup,
    RoomJoin,
    RoomLeave,
}

/// SFU routing mode for a session.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfuMode {
    /// str0m sans-I/O sovereign SFU.
    Sovereign,
    /// `LiveKit` hosted SFU.
    Hosted,
}

/// WebRTC signaling envelope — routed through the spine, never stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalEnvelope {
    pub from_session: SessionId,
    pub to_session: Option<SessionId>,
    pub tenant_id: TenantId,
    /// Room / conference identifier.
    pub room_id: String,
    pub kind: SignalKind,
    pub sfu_mode: SfuMode,
    /// SDP or ICE candidate JSON payload.
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
