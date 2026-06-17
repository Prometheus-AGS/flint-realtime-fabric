use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{ChannelId, EventId, TenantId};

/// Logical channel on the event spine — a topic-like address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub tenant_id: TenantId,
    /// Hierarchical path, e.g. `"entity/user/updates"`.
    pub path: String,
}

/// Monotonic position within a `Channel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Offset(pub u64);

impl Offset {
    pub const BEGINNING: Self = Self(0);

    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// A named, persistent read position for a consumer on a `Channel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cursor {
    pub channel_id: ChannelId,
    pub consumer_id: String,
    pub offset: Offset,
    pub updated_at: DateTime<Utc>,
}

/// Strongly-typed payload kind carried inside `EventEnvelope`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    EntityChange,
    AgentEvent,
    SyncOp,
    Presence,
    Signal,
    Custom(String),
}

/// The universal stamped envelope for every event on the spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub channel: Channel,
    pub offset: Offset,
    pub kind: EventKind,
    /// Serialized inner payload. Kept as raw JSON for zero-copy fan-out.
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for request/reply and saga tracing.
    pub correlation_id: Option<String>,
}

impl EventEnvelope {
    #[must_use]
    pub fn new(
        channel: Channel,
        offset: Offset,
        kind: EventKind,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: EventId::new(),
            channel,
            offset,
            kind,
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }
}
