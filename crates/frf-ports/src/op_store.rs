use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{EntityId, TenantId};

use crate::error::PortError;

/// A CRDT operation that has not yet been confirmed by the server.
///
/// `payload` is engine-agnostic binary (Loro export bytes).
/// `local_seq` is a device-local monotonic counter used for ordering and
/// acknowledgment tracking.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PendingOp {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    /// Opaque CRDT binary delta (engine-specific encoding).
    pub payload: Bytes,
    /// Monotonic device-local sequence number.
    pub local_seq: u64,
}

impl PendingOp {
    #[must_use]
    pub fn new(entity_id: EntityId, tenant_id: TenantId, payload: Bytes, local_seq: u64) -> Self {
        Self {
            entity_id,
            tenant_id,
            payload,
            local_seq,
        }
    }
}

/// On-device durable write-ahead log for outgoing CRDT operations.
///
/// Implemented by `frf-store-redb` for native platforms.
/// Stores unsynced ops until the server acknowledges them.
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait OpStore: Send + Sync + 'static {
    /// Durably append an outgoing op before it is sent to the server.
    async fn queue_op(&self, op: PendingOp) -> Result<(), PortError>;

    /// Return all queued ops for an entity in `local_seq` ascending order.
    async fn drain_pending(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
    ) -> Result<Vec<PendingOp>, PortError>;

    /// Mark all ops with `local_seq <= confirmed_seq` as synced (safe to drop).
    async fn mark_synced(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
        confirmed_seq: u64,
    ) -> Result<(), PortError>;
}

/// Engine-agnostic CRDT merge function injected into `SyncUseCase`.
///
/// Implemented by `LoroDeltaApplier` in `frf-crdt`. Using a trait here
/// keeps `frf-app` free of any `frf-crdt` import (dependency inversion).
pub trait ApplyDelta: Send + Sync + 'static {
    /// Apply `delta` to `existing`, returning the merged encoded state.
    ///
    /// Both slices are engine-specific binary. Returns merged bytes.
    ///
    /// # Errors
    ///
    /// Returns `PortError` if the CRDT engine fails to merge the slices.
    fn apply(&self, existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, PortError>;
}
