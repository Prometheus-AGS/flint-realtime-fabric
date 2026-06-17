use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{EntityId, SessionId, TenantId};

/// The kind of mutation applied to an entity.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    Insert,
    Update,
    Delete,
    Upsert,
}

/// A CDC / CRDT delta describing a change to a domain entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityChange {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub entity_type: String,
    pub op: ChangeOp,
    /// Full entity snapshot (Insert/Upsert) or partial patch (Update).
    pub data: serde_json::Value,
    /// Previous snapshot for optimistic conflict detection.
    pub previous: Option<serde_json::Value>,
    pub session_id: Option<SessionId>,
    pub timestamp: DateTime<Utc>,
    /// Lamport-style version vector for CRDT merge ordering.
    pub version: u64,
}
