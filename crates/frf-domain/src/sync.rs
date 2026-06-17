use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{EntityId, SessionId, TenantId};

/// Operation type in a CRDT delta.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOpKind {
    Insert,
    Delete,
    Update,
    Move,
    Merge,
}

/// A single CRDT operation for offline sync.
///
/// The op is intentionally engine-agnostic at this layer — the `payload`
/// carries the engine-specific encoded delta (Loro bytes or automerge bytes).
/// The engine choice (OPEN DECISION) is resolved in `frf-crdt` (adapter).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncOp {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub session_id: SessionId,
    pub kind: SyncOpKind,
    /// Encoded CRDT delta — engine-specific bytes, base64 in JSON transport.
    pub payload: Vec<u8>,
    /// Lamport clock at the originating device.
    pub lamport: u64,
    pub timestamp: DateTime<Utc>,
}
