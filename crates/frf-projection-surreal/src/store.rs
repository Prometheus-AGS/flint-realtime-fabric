use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{ChangeOp, EntityChange, EntityId, TenantId};
use frf_ports::{
    EntityChangeStream, EntityProjectionSnapshot, EntityStore, PortError, ProjectionApply,
    ProjectionCursor,
};
use surrealdb::{Surreal, engine::any::Any, opt::auth::Root};
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use tracing::instrument;

use crate::error::SurrealProjectionError;
use crate::model::{CursorRow, EntityRow};

const ENTITY_TABLE: &str = "entity_projection";
const CURSOR_TABLE: &str = "entity_projection_cursor";
const CURSOR_ID: &str = "main";
type WatchKey = (TenantId, EntityId);
type WatchSenders = HashMap<WatchKey, broadcast::Sender<EntityChange>>;

#[derive(Clone)]
pub struct SurrealEntityProjection {
    db: Surreal<Any>,
    writer: Arc<Mutex<()>>,
    watchers: Arc<RwLock<WatchSenders>>,
}

impl SurrealEntityProjection {
    /// Connect and authenticate to a `SurrealDB` projection database.
    ///
    /// # Errors
    ///
    /// Returns an adapter error when connection, authentication, database selection,
    /// or schema initialization fails.
    pub async fn connect(
        endpoint: &str,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
    ) -> Result<Self, SurrealProjectionError> {
        let db = surrealdb::engine::any::connect(endpoint).await?;
        db.signin(Root {
            username: username.to_owned(),
            password: password.to_owned(),
        })
        .await?;
        db.use_ns(namespace).use_db(database).await?;
        Self::from_connected(db).await
    }

    /// Wrap a selected client and install the projection schema.
    ///
    /// # Errors
    ///
    /// Returns an adapter error when schema initialization fails.
    pub async fn from_connected(db: Surreal<Any>) -> Result<Self, SurrealProjectionError> {
        db.query(
            "DEFINE TABLE IF NOT EXISTS entity_projection SCHEMAFULL;\
             DEFINE FIELD IF NOT EXISTS entity_id ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS tenant_id ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS entity_type ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS op ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS data_json ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS previous_json ON entity_projection TYPE option<string>;\
             DEFINE FIELD IF NOT EXISTS timestamp ON entity_projection TYPE string;\
             DEFINE FIELD IF NOT EXISTS version ON entity_projection TYPE string;\
             DEFINE TABLE IF NOT EXISTS entity_projection_cursor SCHEMAFULL;\
             DEFINE FIELD IF NOT EXISTS source_epoch ON entity_projection_cursor TYPE string;\
             DEFINE FIELD IF NOT EXISTS commit_lsn ON entity_projection_cursor TYPE string;\
             DEFINE FIELD IF NOT EXISTS transaction_index ON entity_projection_cursor TYPE string;\
             DEFINE FIELD IF NOT EXISTS broker_offset ON entity_projection_cursor TYPE string;",
        )
        .await?
        .check()?;
        Ok(Self {
            db,
            writer: Arc::new(Mutex::new(())),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn record_key(entity_id: EntityId, tenant_id: TenantId) -> String {
        format!("{tenant_id}_{entity_id}")
    }

    async fn notify(&self, change: EntityChange) {
        let key = (change.tenant_id, change.entity_id);
        if let Some(sender) = self.watchers.read().await.get(&key) {
            let _ = sender.send(change);
        }
    }
}

#[async_trait]
impl EntityStore for SurrealEntityProjection {
    #[instrument(name = "port::EntityStore::get_entity", skip(self), fields(%entity_id, %tenant_id))]
    async fn get_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<EntityChange>, PortError> {
        let row: Option<EntityRow> = self
            .db
            .select((ENTITY_TABLE, Self::record_key(entity_id, tenant_id)))
            .await
            .map_err(SurrealProjectionError::Database)?;
        row.map(EntityRow::into_change)
            .transpose()
            .map_err(PortError::from)
    }

    #[instrument(name = "port::EntityStore::watch_entity", skip(self), fields(%entity_id, %tenant_id))]
    async fn watch_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<EntityChangeStream, PortError> {
        let receiver = self
            .watchers
            .write()
            .await
            .entry((tenant_id, entity_id))
            .or_insert_with(|| broadcast::channel(64).0)
            .subscribe();
        let stream = BroadcastStream::new(receiver)
            .filter_map(Result::ok)
            .map(Ok);
        Ok(Box::pin(stream))
    }

    #[instrument(name = "port::EntityStore::apply_projection", skip(self, change), fields(entity_id = %change.entity_id, tenant_id = %change.tenant_id, broker_offset = cursor.broker_offset))]
    async fn apply_projection(
        &self,
        mut change: EntityChange,
        cursor: ProjectionCursor,
    ) -> Result<ProjectionApply, PortError> {
        let _guard = self.writer.lock().await;
        let transaction = self
            .db
            .clone()
            .begin()
            .await
            .map_err(SurrealProjectionError::Database)?;
        let prepared: Result<bool, PortError> = async {
            let checkpoint: Option<CursorRow> = transaction
                .select((CURSOR_TABLE, CURSOR_ID))
                .await
                .map_err(SurrealProjectionError::Database)?;
            if let Some(checkpoint) = checkpoint {
                let checkpoint = checkpoint.into_cursor().map_err(PortError::from)?;
                if checkpoint.source_epoch != cursor.source_epoch {
                    return Err(PortError::NotFound(
                        "resnapshot_required: entity projection source epoch changed".to_owned(),
                    ));
                }
                if cursor.broker_offset <= checkpoint.broker_offset {
                    return Ok(false);
                }
            } else if cursor.broker_offset != 0 {
                return Err(PortError::NotFound(
                    "resnapshot_required: retained projection history does not begin at zero"
                        .to_owned(),
                ));
            }

            let key = Self::record_key(change.entity_id, change.tenant_id);
            let previous: Option<EntityRow> = transaction
                .select((ENTITY_TABLE, key.as_str()))
                .await
                .map_err(SurrealProjectionError::Database)?;
            let previous = previous
                .map(EntityRow::into_change)
                .transpose()
                .map_err(PortError::from)?;
            change.previous = previous.as_ref().map(|entity| entity.data.clone());
            if change.op == ChangeOp::Update {
                let existing = previous.as_ref().ok_or_else(|| {
                    PortError::NotFound(
                        "resnapshot_required: update has no projected base row".to_owned(),
                    )
                })?;
                merge_object(&mut change.data, &existing.data)?;
            }
            if change.op == ChangeOp::Delete {
                let _: Option<EntityRow> = transaction
                    .delete((ENTITY_TABLE, key.as_str()))
                    .await
                    .map_err(SurrealProjectionError::Database)?;
            } else {
                let row = EntityRow::from_change(&change).map_err(PortError::from)?;
                let _: Option<EntityRow> = transaction
                    .upsert((ENTITY_TABLE, key.as_str()))
                    .content(row)
                    .await
                    .map_err(SurrealProjectionError::Database)?;
            }
            let _: Option<CursorRow> = transaction
                .upsert((CURSOR_TABLE, CURSOR_ID))
                .content(CursorRow::from_cursor(&cursor))
                .await
                .map_err(SurrealProjectionError::Database)?;
            Ok(true)
        }
        .await;
        match prepared {
            Ok(true) => {}
            Ok(false) => {
                transaction
                    .cancel()
                    .await
                    .map_err(SurrealProjectionError::Database)?;
                return Ok(ProjectionApply::Duplicate);
            }
            Err(error) => {
                transaction
                    .cancel()
                    .await
                    .map_err(SurrealProjectionError::Database)?;
                return Err(error);
            }
        }
        transaction
            .commit()
            .await
            .map_err(SurrealProjectionError::Database)?;
        self.notify(change.clone()).await;
        Ok(ProjectionApply::Applied(Box::new(change)))
    }

    #[instrument(name = "port::EntityStore::install_projection_snapshot", skip(self, snapshot), fields(broker_offset = snapshot.cursor.broker_offset))]
    async fn install_projection_snapshot(
        &self,
        snapshot: EntityProjectionSnapshot,
    ) -> Result<bool, PortError> {
        let _guard = self.writer.lock().await;
        if self
            .projection_checkpoint()
            .await?
            .is_some_and(|checkpoint| {
                checkpoint.source_epoch == snapshot.cursor.source_epoch
                    && checkpoint.broker_offset >= snapshot.cursor.broker_offset
            })
        {
            return Ok(false);
        }
        let transaction = self
            .db
            .clone()
            .begin()
            .await
            .map_err(SurrealProjectionError::Database)?;
        let prepared: Result<(), PortError> = async {
            let _: Vec<EntityRow> = transaction
                .delete(ENTITY_TABLE)
                .await
                .map_err(SurrealProjectionError::Database)?;
            for entity in &snapshot.entities {
                let key = Self::record_key(entity.entity_id, entity.tenant_id);
                let row = EntityRow::from_change(entity).map_err(PortError::from)?;
                let _: Option<EntityRow> = transaction
                    .upsert((ENTITY_TABLE, key.as_str()))
                    .content(row)
                    .await
                    .map_err(SurrealProjectionError::Database)?;
            }
            let _: Option<CursorRow> = transaction
                .upsert((CURSOR_TABLE, CURSOR_ID))
                .content(CursorRow::from_cursor(&snapshot.cursor))
                .await
                .map_err(SurrealProjectionError::Database)?;
            Ok(())
        }
        .await;
        if let Err(error) = prepared {
            transaction
                .cancel()
                .await
                .map_err(SurrealProjectionError::Database)?;
            return Err(error);
        }
        transaction
            .commit()
            .await
            .map_err(SurrealProjectionError::Database)?;
        Ok(true)
    }

    async fn projection_checkpoint(&self) -> Result<Option<ProjectionCursor>, PortError> {
        let row: Option<CursorRow> = self
            .db
            .select((CURSOR_TABLE, CURSOR_ID))
            .await
            .map_err(SurrealProjectionError::Database)?;
        row.map(CursorRow::into_cursor)
            .transpose()
            .map_err(PortError::from)
    }
}

fn merge_object(
    patch: &mut serde_json::Value,
    existing: &serde_json::Value,
) -> Result<(), PortError> {
    let (Some(patch), Some(existing)) = (patch.as_object_mut(), existing.as_object()) else {
        return Err(PortError::Serialization(
            "entity update and projected base must be JSON objects".to_owned(),
        ));
    };
    for (key, value) in existing {
        patch.entry(key.clone()).or_insert_with(|| value.clone());
    }
    Ok(())
}
