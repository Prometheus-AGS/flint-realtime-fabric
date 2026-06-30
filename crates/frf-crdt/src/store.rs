use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{EntityId, TenantId};
use frf_ports::{CrdtSnapshot, CrdtStore, PortError};
use loro::LoroDoc;
use tracing::instrument;

use crate::error::CrdtError;
use crate::merge::apply_delta;

/// In-memory CRDT store backed by Loro.
///
/// Suitable for tests and ephemeral single-process usage. Production
/// deployments wire `frf-store-surreal`'s `SurrealCrdtStore` instead.
#[derive(Debug, Default)]
pub struct InMemoryCrdtStore {
    inner: std::sync::Mutex<std::collections::HashMap<StoreKey, CrdtSnapshot>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StoreKey {
    entity_id: EntityId,
    tenant_id: TenantId,
}

#[async_trait]
impl CrdtStore for InMemoryCrdtStore {
    #[instrument(skip(self), fields(entity_id = %snapshot.entity_id, tenant_id = %snapshot.tenant_id))]
    async fn checkpoint(&self, snapshot: CrdtSnapshot) -> Result<(), PortError> {
        let key = StoreKey {
            entity_id: snapshot.entity_id,
            tenant_id: snapshot.tenant_id,
        };
        let mut map = self
            .inner
            .lock()
            .map_err(|_| PortError::Transport("lock poisoned".into()))?;
        map.insert(key, snapshot);
        Ok(())
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    async fn restore(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<CrdtSnapshot>, PortError> {
        let key = StoreKey {
            entity_id,
            tenant_id,
        };
        let map = self
            .inner
            .lock()
            .map_err(|_| PortError::Transport("lock poisoned".into()))?;
        Ok(map.get(&key).cloned())
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    async fn purge(&self, entity_id: EntityId, tenant_id: TenantId) -> Result<(), PortError> {
        let key = StoreKey {
            entity_id,
            tenant_id,
        };
        let mut map = self
            .inner
            .lock()
            .map_err(|_| PortError::Transport("lock poisoned".into()))?;
        map.remove(&key);
        Ok(())
    }
}

/// Apply a Loro delta into the snapshot stored for `entity_id`/`tenant_id`,
/// then persist the resulting snapshot back to the store.
///
/// Convenience function used by `SyncUseCase`; keeps merge logic in one place.
///
/// # Errors
///
/// Returns `CrdtError` if restoring, merging, or checkpointing the snapshot fails.
#[instrument(skip(store, delta_bytes), fields(%entity_id, %tenant_id, delta_len = delta_bytes.len()))]
pub async fn merge_into_store<S: CrdtStore>(
    store: &S,
    entity_id: EntityId,
    tenant_id: TenantId,
    delta_bytes: &[u8],
    new_version: u64,
) -> Result<CrdtSnapshot, CrdtError> {
    let existing = store
        .restore(entity_id, tenant_id)
        .await?
        .map(|s| s.encoded.to_vec())
        .unwrap_or_default();

    let merged = apply_delta(&existing, delta_bytes)?;

    let snapshot = CrdtSnapshot {
        entity_id,
        tenant_id,
        encoded: Bytes::from(merged),
        version: new_version,
    };

    store.checkpoint(snapshot.clone()).await?;
    Ok(snapshot)
}

/// Export the current snapshot for `entity_id` as a Loro all-updates delta.
///
/// Returns `None` when no checkpoint exists yet.
///
/// # Errors
///
/// Returns `CrdtError` if restoring the snapshot or exporting the Loro update fails.
#[instrument(skip(store), fields(%entity_id, %tenant_id))]
pub async fn export_updates_since<S: CrdtStore>(
    store: &S,
    entity_id: EntityId,
    tenant_id: TenantId,
    since_version: u64,
) -> Result<Option<Vec<u8>>, CrdtError> {
    let Some(snapshot) = store.restore(entity_id, tenant_id).await? else {
        return Ok(None);
    };

    let doc = LoroDoc::new();
    doc.import(&snapshot.encoded)
        .map_err(|e| CrdtError::Decode(e.to_string()))?;

    let updates = doc
        .export(loro::ExportMode::all_updates())
        .map_err(|e| CrdtError::Encode(e.to_string()))?;

    Ok(Some(updates))
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_domain::{EntityId, TenantId};
    use loro::LoroDoc;

    fn entity() -> EntityId {
        EntityId::new()
    }

    fn tenant() -> TenantId {
        TenantId::new()
    }

    fn snapshot_bytes(key: &str, val: &str) -> Bytes {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert(key, val).unwrap();
        Bytes::from(doc.export(loro::ExportMode::Snapshot).unwrap())
    }

    #[tokio::test]
    async fn checkpoint_and_restore_roundtrip() {
        let store = InMemoryCrdtStore::default();
        let eid = entity();
        let tid = tenant();
        let snap = CrdtSnapshot {
            entity_id: eid,
            tenant_id: tid,
            encoded: snapshot_bytes("k", "v"),
            version: 1,
        };
        store.checkpoint(snap.clone()).await.unwrap();
        let restored = store.restore(eid, tid).await.unwrap();
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().version, 1);
    }

    #[tokio::test]
    async fn restore_returns_none_for_unknown_entity() {
        let store = InMemoryCrdtStore::default();
        let result = store.restore(entity(), tenant()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn purge_removes_snapshot() {
        let store = InMemoryCrdtStore::default();
        let eid = entity();
        let tid = tenant();
        let snap = CrdtSnapshot {
            entity_id: eid,
            tenant_id: tid,
            encoded: snapshot_bytes("k", "v"),
            version: 1,
        };
        store.checkpoint(snap).await.unwrap();
        store.purge(eid, tid).await.unwrap();
        assert!(store.restore(eid, tid).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn merge_into_store_accumulates_keys() {
        let store = InMemoryCrdtStore::default();
        let eid = entity();
        let tid = tenant();

        let doc_a = LoroDoc::new();
        doc_a.get_map("root").insert("a", "1").unwrap();
        let base = doc_a.export(loro::ExportMode::Snapshot).unwrap();

        let doc_b = LoroDoc::new();
        doc_b.get_map("root").insert("b", "2").unwrap();
        let delta = doc_b.export(loro::ExportMode::all_updates()).unwrap();

        // Seed base
        store
            .checkpoint(CrdtSnapshot {
                entity_id: eid,
                tenant_id: tid,
                encoded: Bytes::from(base),
                version: 1,
            })
            .await
            .unwrap();

        let merged = merge_into_store(&store, eid, tid, &delta, 2).await.unwrap();

        let doc = LoroDoc::new();
        doc.import(&merged.encoded).unwrap();
        let map = doc.get_map("root");
        let read_str = |map: &loro::LoroMap, key: &str| -> Option<String> {
            let voc = map.get(key)?;
            match voc.into_value() {
                Ok(loro::LoroValue::String(s)) => Some(s.to_string()),
                _ => None,
            }
        };
        assert_eq!(read_str(&map, "a").as_deref(), Some("1"));
        assert_eq!(read_str(&map, "b").as_deref(), Some("2"));
        assert_eq!(merged.version, 2);
    }
}
