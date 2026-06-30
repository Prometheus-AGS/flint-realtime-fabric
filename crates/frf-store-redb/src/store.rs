use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{EntityId, TenantId};
use frf_ports::{OpStore, PendingOp, PortError};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use tracing::instrument;

use crate::error::RedbOpStoreError;
use crate::key::{KEY_LEN, make_key, make_prefix, make_prefix_end, seq_from_key};

/// Table: `composite_key([u8; 40])` → `payload([u8])`
const OPS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("pending_ops");

/// On-device durable op-log backed by redb.
///
/// Thread-safe: `Database` is wrapped in `Arc` and redb supports concurrent
/// read transactions with a single write transaction at a time.
///
/// Intended for native (non-WASM) targets. The `frf-ffi` crate exposes a
/// synchronous wrapper for Swift/Kotlin callers.
#[derive(Clone)]
pub struct RedbOpStore {
    db: Arc<Database>,
}

impl std::fmt::Debug for RedbOpStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbOpStore").finish_non_exhaustive()
    }
}

impl RedbOpStore {
    /// Open (or create) the redb database at `path`.
    ///
    /// Creates the `pending_ops` table on first open.
    ///
    /// # Errors
    ///
    /// Returns `RedbOpStoreError` if the database cannot be created or the
    /// initial write transaction fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RedbOpStoreError> {
        let db = Database::create(path.as_ref())?;
        {
            let wtx = db.begin_write()?;
            // Ensure the table exists.
            let _ = wtx.open_table(OPS)?;
            wtx.commit()?;
        }
        Ok(Self { db: Arc::new(db) })
    }

    /// Open an in-memory redb instance (useful for tests without tempfile).
    ///
    /// # Errors
    ///
    /// Returns `RedbOpStoreError` if database or table initialization fails.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, RedbOpStoreError> {
        use redb::backends::InMemoryBackend;
        let db = Database::builder().create_with_backend(InMemoryBackend::new())?;
        {
            let wtx = db.begin_write()?;
            let _ = wtx.open_table(OPS)?;
            wtx.commit()?;
        }
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl OpStore for RedbOpStore {
    #[instrument(skip(self, op), fields(entity_id = %op.entity_id, tenant_id = %op.tenant_id, local_seq = op.local_seq))]
    async fn queue_op(&self, op: PendingOp) -> Result<(), PortError> {
        let key = make_key(&op.entity_id, &op.tenant_id, op.local_seq);
        let payload = op.payload.to_vec();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let wtx = db.begin_write().map_err(RedbOpStoreError::Transaction)?;
            {
                let mut table = wtx.open_table(OPS).map_err(RedbOpStoreError::Table)?;
                table
                    .insert(key.as_ref(), payload.as_slice())
                    .map_err(RedbOpStoreError::Storage)?;
            }
            wtx.commit().map_err(RedbOpStoreError::Commit)?;
            Ok::<_, RedbOpStoreError>(())
        })
        .await
        .map_err(|e| PortError::Transport(format!("spawn_blocking: {e}")))?
        .map_err(PortError::from)
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    async fn drain_pending(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
    ) -> Result<Vec<PendingOp>, PortError> {
        let prefix_start = make_prefix(entity_id, tenant_id);
        let prefix_end = make_prefix_end(entity_id, tenant_id);
        let eid = *entity_id;
        let tid = *tenant_id;
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let rtx = db.begin_read().map_err(RedbOpStoreError::Transaction)?;
            let table = rtx.open_table(OPS).map_err(RedbOpStoreError::Table)?;

            let range = table
                .range(prefix_start.as_ref()..prefix_end.as_ref())
                .map_err(RedbOpStoreError::Storage)?;

            let mut ops = Vec::new();
            for entry in range {
                let (k, v) = entry.map_err(RedbOpStoreError::Storage)?;
                let key_bytes = k.value();
                if key_bytes.len() < KEY_LEN {
                    continue;
                }
                let local_seq = seq_from_key(key_bytes);
                let payload = Bytes::copy_from_slice(v.value());
                ops.push(PendingOp::new(eid, tid, payload, local_seq));
            }
            Ok::<_, RedbOpStoreError>(ops)
        })
        .await
        .map_err(|e| PortError::Transport(format!("spawn_blocking: {e}")))?
        .map_err(PortError::from)
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id, confirmed_seq))]
    async fn mark_synced(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
        confirmed_seq: u64,
    ) -> Result<(), PortError> {
        let prefix_start = make_prefix(entity_id, tenant_id);
        let prefix_end = make_prefix_end(entity_id, tenant_id);
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let wtx = db.begin_write().map_err(RedbOpStoreError::Transaction)?;
            {
                let mut table = wtx.open_table(OPS).map_err(RedbOpStoreError::Table)?;

                // Collect keys to delete first (can't mutate during iteration).
                let keys_to_delete: Vec<Vec<u8>> = {
                    let read = table
                        .range(prefix_start.as_ref()..prefix_end.as_ref())
                        .map_err(RedbOpStoreError::Storage)?;
                    read.filter_map(|entry| {
                        let (k, _) = entry.ok()?;
                        let key_bytes = k.value();
                        if key_bytes.len() < KEY_LEN {
                            return None;
                        }
                        let seq = seq_from_key(key_bytes);
                        if seq <= confirmed_seq {
                            Some(key_bytes.to_vec())
                        } else {
                            None
                        }
                    })
                    .collect()
                };

                for key in keys_to_delete {
                    table
                        .remove(key.as_slice())
                        .map_err(RedbOpStoreError::Storage)?;
                }
            }
            wtx.commit().map_err(RedbOpStoreError::Commit)?;
            Ok::<_, RedbOpStoreError>(())
        })
        .await
        .map_err(|e| PortError::Transport(format!("spawn_blocking: {e}")))?
        .map_err(PortError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn eid() -> EntityId {
        EntityId::new()
    }

    fn tid() -> TenantId {
        TenantId::new()
    }

    fn op(eid: EntityId, tid: TenantId, seq: u64) -> PendingOp {
        PendingOp::new(eid, tid, Bytes::from(format!("payload-{seq}")), seq)
    }

    #[tokio::test]
    async fn queue_and_drain() {
        let store = RedbOpStore::in_memory().unwrap();
        let e = eid();
        let t = tid();

        store.queue_op(op(e, t, 1)).await.unwrap();
        store.queue_op(op(e, t, 2)).await.unwrap();
        store.queue_op(op(e, t, 3)).await.unwrap();

        let ops = store.drain_pending(&e, &t).await.unwrap();
        assert_eq!(ops.len(), 3);
        // Must be in ascending seq order.
        assert_eq!(ops[0].local_seq, 1);
        assert_eq!(ops[1].local_seq, 2);
        assert_eq!(ops[2].local_seq, 3);
    }

    #[tokio::test]
    async fn mark_synced_removes_confirmed() {
        let store = RedbOpStore::in_memory().unwrap();
        let e = eid();
        let t = tid();

        for seq in 1..=5 {
            store.queue_op(op(e, t, seq)).await.unwrap();
        }

        store.mark_synced(&e, &t, 3).await.unwrap();

        let remaining = store.drain_pending(&e, &t).await.unwrap();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].local_seq, 4);
        assert_eq!(remaining[1].local_seq, 5);
    }

    #[tokio::test]
    async fn drain_empty_returns_empty_vec() {
        let store = RedbOpStore::in_memory().unwrap();
        let e = eid();
        let t = tid();
        let ops = store.drain_pending(&e, &t).await.unwrap();
        assert!(ops.is_empty());
    }

    #[tokio::test]
    async fn different_entities_are_isolated() {
        let store = RedbOpStore::in_memory().unwrap();
        let e1 = eid();
        let e2 = eid();
        let t = tid();

        store.queue_op(op(e1, t, 1)).await.unwrap();
        store.queue_op(op(e2, t, 1)).await.unwrap();

        let ops1 = store.drain_pending(&e1, &t).await.unwrap();
        let ops2 = store.drain_pending(&e2, &t).await.unwrap();
        assert_eq!(ops1.len(), 1);
        assert_eq!(ops2.len(), 1);
    }
}
