use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{EntityId, TenantId};

use crate::error::PortError;

/// A versioned CRDT snapshot: encoded bytes + version vector.
#[derive(Debug, Clone)]
pub struct CrdtSnapshot {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    /// Engine-specific encoded state (Loro or automerge — resolved in Phase 3).
    pub encoded: Bytes,
    pub version: u64,
}

/// Checkpoint and restore CRDT state across server restarts or device sync.
///
/// Implemented by `frf-store-surreal` (`SurrealDB` 3.x).
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait CrdtStore: Send + Sync + 'static {
    /// Persist a CRDT snapshot for an entity.
    async fn checkpoint(&self, snapshot: CrdtSnapshot) -> Result<(), PortError>;

    /// Restore the latest CRDT snapshot for an entity, if any.
    async fn restore(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<CrdtSnapshot>, PortError>;

    /// Delete all snapshots for an entity (e.g., on hard delete).
    async fn purge(&self, entity_id: EntityId, tenant_id: TenantId) -> Result<(), PortError>;
}
