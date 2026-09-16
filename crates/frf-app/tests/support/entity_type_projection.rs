use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{
    ChangeOp, EntityChange, EntityId, EntityTypeDelivery, EntityTypeSelector, TenantId,
};
use frf_ports::{
    EntityChangeStream, EntityProjectionSnapshot, EntityStore, EntityTypeProjectionSnapshot,
    PortError, ProjectionApply, ProjectionCursor,
};
use futures_util::stream;
use tokio::sync::Mutex;

pub struct TestProjection {
    history: Arc<Mutex<Vec<EntityTypeDelivery>>>,
}

impl TestProjection {
    pub fn new(history: Arc<Mutex<Vec<EntityTypeDelivery>>>) -> Self {
        Self { history }
    }
}

#[async_trait]
impl EntityStore for TestProjection {
    async fn get_entity(
        &self,
        _entity_id: EntityId,
        _tenant_id: TenantId,
    ) -> Result<Option<EntityChange>, PortError> {
        Ok(None)
    }

    async fn watch_entity(
        &self,
        _entity_id: EntityId,
        _tenant_id: TenantId,
    ) -> Result<EntityChangeStream, PortError> {
        Ok(Box::pin(stream::empty()))
    }

    async fn apply_projection(
        &self,
        _change: EntityChange,
        _cursor: ProjectionCursor,
    ) -> Result<ProjectionApply, PortError> {
        Err(PortError::Transport(
            "test projection is read only".to_owned(),
        ))
    }

    async fn install_projection_snapshot(
        &self,
        _snapshot: EntityProjectionSnapshot,
    ) -> Result<bool, PortError> {
        Err(PortError::Transport(
            "test projection is read only".to_owned(),
        ))
    }

    async fn projection_checkpoint(&self) -> Result<Option<ProjectionCursor>, PortError> {
        Ok(cursor(self.history.lock().await.last()))
    }

    async fn snapshot_entity_type(
        &self,
        _entity_type: &EntityTypeSelector,
        _tenant_id: TenantId,
    ) -> Result<EntityTypeProjectionSnapshot, PortError> {
        let history = self.history.lock().await;
        let mut rows = HashMap::new();
        // Deliberately return every scope so the application-layer isolation
        // guard is tested independently from projection query correctness.
        for delivery in history.iter() {
            let key = delivery.mutation.key.canonical_id.clone();
            if delivery.mutation.op == ChangeOp::Delete {
                rows.remove(&key);
            } else {
                rows.insert(key, delivery.clone());
            }
        }
        let mut entities = rows.into_values().collect::<Vec<_>>();
        entities.sort_by(|left, right| {
            left.mutation
                .key
                .canonical_id
                .cmp(&right.mutation.key.canonical_id)
        });
        Ok(EntityTypeProjectionSnapshot {
            entities,
            cursor: cursor(history.last()),
        })
    }
}

fn cursor(delivery: Option<&EntityTypeDelivery>) -> Option<ProjectionCursor> {
    delivery.map(|delivery| ProjectionCursor {
        source_epoch: delivery.mutation.source.epoch.clone(),
        commit_lsn: delivery.mutation.source.commit_lsn,
        transaction_index: delivery.mutation.source.transaction_index,
        broker_offset: delivery.broker_offset,
    })
}
