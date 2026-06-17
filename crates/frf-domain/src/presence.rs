use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{ChannelId, SessionId, TenantId};

/// Online/offline status of a session.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
    Busy,
}

/// User/device presence record broadcast on a channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Presence {
    pub session_id: SessionId,
    pub tenant_id: TenantId,
    pub channel_id: ChannelId,
    pub user_id: String,
    pub display_name: Option<String>,
    pub status: PresenceStatus,
    /// Arbitrary metadata (cursor position, selected entity, etc.).
    pub meta: serde_json::Value,
    pub last_seen: DateTime<Utc>,
    /// Heartbeat expiry — server clears presence after this time with no ping.
    pub expires_at: DateTime<Utc>,
}
