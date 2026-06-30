use bytes::Bytes;
use frf_domain::{EntityId, TenantId};
use frf_ports::{ApplyDelta, CrdtSnapshot, CrdtStore, OpStore, PendingOp, PortError};
use tracing::instrument;

/// Request to apply an incoming server delta to the local CRDT state.
#[derive(Debug, Clone)]
pub struct SyncRequest {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    /// Raw delta bytes from the server (engine-specific encoding).
    pub delta: Bytes,
    /// Server-confirmed sequence number up to which pending ops can be dropped.
    pub confirmed_seq: u64,
    /// New version after applying the delta.
    pub new_version: u64,
}

/// Response from a successful sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    /// Merged CRDT snapshot bytes after applying `delta`.
    pub snapshot: Bytes,
    pub version: u64,
    /// How many pending ops remain after dropping confirmed ones.
    pub remaining_ops: usize,
}

/// Application-layer use-case for offline-first CRDT sync.
///
/// Wires together three port traits:
/// - `S: CrdtStore` — reads/writes the merged snapshot (server-side or in-memory).
/// - `O: OpStore`   — drains and acknowledges pending outgoing ops.
/// - `A: ApplyDelta`— merges engine-specific bytes without importing `frf-crdt`.
///
/// No adapter crate (`frf-crdt`, `frf-store-*`) is imported here; the
/// dependency inversion is enforced at the Cargo level by `frf-app`'s
/// `[dependencies]` section.
pub struct SyncUseCase<S, O, A> {
    crdt_store: S,
    op_store: O,
    applier: A,
}

impl<S, O, A> SyncUseCase<S, O, A>
where
    S: CrdtStore,
    O: OpStore,
    A: ApplyDelta,
{
    pub fn new(crdt_store: S, op_store: O, applier: A) -> Self {
        Self {
            crdt_store,
            op_store,
            applier,
        }
    }

    /// Apply a server delta, persist the merged snapshot, and acknowledge ops.
    ///
    /// Steps:
    /// 1. Restore the current snapshot from `CrdtStore` (or empty if none).
    /// 2. Merge `delta` into the snapshot via `ApplyDelta`.
    /// 3. Persist the merged snapshot back to `CrdtStore`.
    /// 4. Mark all pending ops with `local_seq <= confirmed_seq` as synced.
    /// 5. Return the merged state and remaining op count.
    ///
    /// # Errors
    ///
    /// Returns `PortError` if any storage or CRDT merge operation fails.
    #[instrument(skip(self, request), fields(
        entity_id = %request.entity_id,
        tenant_id = %request.tenant_id,
        confirmed_seq = request.confirmed_seq,
        new_version = request.new_version,
    ))]
    pub async fn apply_server_delta(&self, request: SyncRequest) -> Result<SyncResult, PortError> {
        let SyncRequest {
            entity_id,
            tenant_id,
            delta,
            confirmed_seq,
            new_version,
        } = request;

        // 1. Restore existing snapshot.
        let existing = self
            .crdt_store
            .restore(entity_id, tenant_id)
            .await?
            .map(|s| s.encoded.to_vec())
            .unwrap_or_default();

        // 2. Merge.
        let merged_bytes = self.applier.apply(&existing, &delta)?;

        // 3. Checkpoint merged state.
        let merged = Bytes::from(merged_bytes);
        self.crdt_store
            .checkpoint(CrdtSnapshot {
                entity_id,
                tenant_id,
                encoded: merged.clone(),
                version: new_version,
            })
            .await?;

        // 4. Acknowledge confirmed ops.
        self.op_store
            .mark_synced(&entity_id, &tenant_id, confirmed_seq)
            .await?;

        // 5. Count remaining pending ops.
        let remaining = self.op_store.drain_pending(&entity_id, &tenant_id).await?;

        Ok(SyncResult {
            entity_id,
            tenant_id,
            snapshot: merged,
            version: new_version,
            remaining_ops: remaining.len(),
        })
    }

    /// Queue an outgoing local op before it is sent to the server.
    ///
    /// The caller is responsible for assigning a monotonically increasing
    /// `local_seq` within the device scope.
    ///
    /// # Errors
    ///
    /// Returns `PortError` if the op-store write fails.
    #[instrument(skip(self, op), fields(
        entity_id = %op.entity_id,
        tenant_id = %op.tenant_id,
        local_seq = op.local_seq,
    ))]
    pub async fn queue_local_op(&self, op: PendingOp) -> Result<(), PortError> {
        self.op_store.queue_op(op).await
    }

    /// Return all pending outgoing ops for the given entity, in ascending seq order.
    ///
    /// # Errors
    ///
    /// Returns `PortError` if the op-store read fails.
    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    pub async fn pending_ops(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
    ) -> Result<Vec<PendingOp>, PortError> {
        self.op_store.drain_pending(entity_id, tenant_id).await
    }

    /// Return the current CRDT snapshot for an entity, or `None` if not yet checkpointed.
    ///
    /// # Errors
    ///
    /// Returns `PortError` if the CRDT-store read fails.
    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    pub async fn snapshot(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<CrdtSnapshot>, PortError> {
        self.crdt_store.restore(entity_id, tenant_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use frf_ports::{CrdtSnapshot, PortError};
    use mockall::mock;
    use mockall::predicate::*;

    // ── mock CrdtStore ────────────────────────────────────────────────────────
    mock! {
        pub CrdtStoreMock {}

        #[async_trait::async_trait]
        impl CrdtStore for CrdtStoreMock {
            async fn checkpoint(&self, snapshot: CrdtSnapshot) -> Result<(), PortError>;
            async fn restore(
                &self,
                entity_id: EntityId,
                tenant_id: TenantId,
            ) -> Result<Option<CrdtSnapshot>, PortError>;
            async fn purge(&self, entity_id: EntityId, tenant_id: TenantId) -> Result<(), PortError>;
        }
    }

    // ── mock OpStore ──────────────────────────────────────────────────────────
    mock! {
        pub OpStoreMock {}

        #[async_trait::async_trait]
        impl OpStore for OpStoreMock {
            async fn queue_op(&self, op: PendingOp) -> Result<(), PortError>;
            async fn drain_pending(
                &self,
                entity_id: &EntityId,
                tenant_id: &TenantId,
            ) -> Result<Vec<PendingOp>, PortError>;
            async fn mark_synced(
                &self,
                entity_id: &EntityId,
                tenant_id: &TenantId,
                confirmed_seq: u64,
            ) -> Result<(), PortError>;
        }
    }

    // ── stub ApplyDelta ───────────────────────────────────────────────────────
    struct PassthroughApplier;
    impl ApplyDelta for PassthroughApplier {
        fn apply(&self, _existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, PortError> {
            Ok(delta.to_vec())
        }
    }

    fn eid() -> EntityId {
        EntityId::new()
    }
    fn tid() -> TenantId {
        TenantId::new()
    }

    #[tokio::test]
    async fn apply_server_delta_checkpoints_and_acks() {
        let e = eid();
        let t = tid();

        let mut crdt = MockCrdtStoreMock::new();
        crdt.expect_restore().returning(|_, _| Ok(None));
        crdt.expect_checkpoint().returning(|_| Ok(()));

        let mut ops = MockOpStoreMock::new();
        ops.expect_mark_synced().returning(|_, _, _| Ok(()));
        ops.expect_drain_pending().returning(|_, _| Ok(vec![]));

        let uc = SyncUseCase::new(crdt, ops, PassthroughApplier);
        let req = SyncRequest {
            entity_id: e,
            tenant_id: t,
            delta: Bytes::from_static(b"delta-bytes"),
            confirmed_seq: 5,
            new_version: 2,
        };

        let result = uc.apply_server_delta(req).await.unwrap();
        assert_eq!(result.snapshot.as_ref(), b"delta-bytes");
        assert_eq!(result.version, 2);
        assert_eq!(result.remaining_ops, 0);
    }

    #[tokio::test]
    async fn queue_local_op_delegates_to_op_store() {
        let e = eid();
        let t = tid();

        let crdt = MockCrdtStoreMock::new();
        let mut ops = MockOpStoreMock::new();
        ops.expect_queue_op().returning(|_| Ok(()));

        let uc = SyncUseCase::new(crdt, ops, PassthroughApplier);
        let op = PendingOp::new(e, t, Bytes::from_static(b"op"), 1);

        uc.queue_local_op(op).await.unwrap();
    }
}
