use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{EntityChange, EntityId, TenantId};
use frf_ports::{EntityChangeStream, EntityStore, PortError};
use tokio::sync::{RwLock, broadcast};
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

/// In-memory `EntityStore` used as the gateway's default wiring for the entity plane.
///
/// Persistent deployments swap in a real adapter (e.g. a future `frf-store-surreal`
/// `EntityStore` impl). This holds the latest change per `(tenant, entity)` and a
/// per-entity broadcast channel so `watch_entity` streams subsequent `put`s. It is the
/// entity-plane analogue of the `InMemoryCrdtStore` used for `SyncService`.
#[derive(Clone, Default)]
pub struct InMemoryEntityStore {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    latest: HashMap<(TenantId, EntityId), EntityChange>,
    watchers: HashMap<(TenantId, EntityId), broadcast::Sender<EntityChange>>,
}

impl InMemoryEntityStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a change: update the latest snapshot and notify any watchers.
    /// Used by ingestion paths (e.g. CDC) and by tests.
    pub async fn put(&self, change: EntityChange) {
        let key = (change.tenant_id, change.entity_id);
        let mut inner = self.inner.write().await;
        inner.latest.insert(key, change.clone());
        if let Some(tx) = inner.watchers.get(&key) {
            // A send error means no active receivers — safe to ignore.
            let _ = tx.send(change);
        }
    }
}

#[async_trait]
impl EntityStore for InMemoryEntityStore {
    async fn get_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<EntityChange>, PortError> {
        let inner = self.inner.read().await;
        Ok(inner.latest.get(&(tenant_id, entity_id)).cloned())
    }

    async fn watch_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<EntityChangeStream, PortError> {
        let key = (tenant_id, entity_id);
        let rx = {
            let mut inner = self.inner.write().await;
            inner
                .watchers
                .entry(key)
                .or_insert_with(|| broadcast::channel(64).0)
                .subscribe()
        };
        // Drop lagged/closed frames rather than surfacing them as stream errors.
        let stream = BroadcastStream::new(rx).filter_map(Result::ok).map(Ok);
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use frf_domain::ChangeOp;

    fn sample(entity: EntityId, tenant: TenantId, version: u64) -> EntityChange {
        EntityChange {
            entity_id: entity,
            tenant_id: tenant,
            entity_type: "widget".to_owned(),
            op: ChangeOp::Upsert,
            data: serde_json::json!({ "n": version }),
            previous: None,
            session_id: None,
            timestamp: Utc::now(),
            version,
        }
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_entity() {
        let store = InMemoryEntityStore::new();
        let got = store
            .get_entity(EntityId::new(), TenantId::new())
            .await
            .expect("get should not error");
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn put_then_get_returns_latest() {
        let store = InMemoryEntityStore::new();
        let (e, t) = (EntityId::new(), TenantId::new());
        store.put(sample(e, t, 1)).await;
        store.put(sample(e, t, 2)).await;

        let got = store.get_entity(e, t).await.expect("get").expect("some");
        assert_eq!(got.version, 2);
    }

    #[tokio::test]
    async fn get_is_tenant_scoped() {
        let store = InMemoryEntityStore::new();
        let e = EntityId::new();
        let (t1, t2) = (TenantId::new(), TenantId::new());
        store.put(sample(e, t1, 1)).await;

        // Same entity id, different tenant → not visible.
        let got = store.get_entity(e, t2).await.expect("get");
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn watch_streams_subsequent_puts() {
        let store = InMemoryEntityStore::new();
        let (e, t) = (EntityId::new(), TenantId::new());
        let mut stream = store.watch_entity(e, t).await.expect("watch");

        store.put(sample(e, t, 7)).await;
        let next = stream.next().await.expect("item").expect("ok");
        assert_eq!(next.version, 7);
    }
}
