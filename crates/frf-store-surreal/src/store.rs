use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{EntityId, TenantId};
use frf_ports::{CrdtSnapshot, CrdtStore, PortError};
use surrealdb::{Surreal, engine::any::Any};
use tracing::instrument;

use crate::error::SurrealStoreError;
use crate::model::SnapshotRow;

const TABLE: &str = "crdt_snapshots";

/// SurrealDB-backed CRDT checkpoint store.
///
/// One `SurrealCrdtStore` is wired per deployment in `frf-gateway`. The caller
/// is responsible for calling `connect` + `use_ns` + `use_db` before passing
/// the client to this adapter.
///
/// Schema (auto-created on first write — no migration required):
/// ```sql
/// DEFINE TABLE crdt_snapshots SCHEMAFULL;
/// DEFINE FIELD entity_id   ON crdt_snapshots TYPE string;
/// DEFINE FIELD tenant_id   ON crdt_snapshots TYPE string;
/// DEFINE FIELD encoded_hex ON crdt_snapshots TYPE string;
/// DEFINE FIELD version     ON crdt_snapshots TYPE int;
/// DEFINE INDEX idx_entity_tenant ON crdt_snapshots
///     COLUMNS entity_id, tenant_id UNIQUE;
/// ```
/// The `UNIQUE` index means `checkpoint` is an UPSERT on the `(entity_id,
/// tenant_id)` pair — only one snapshot per entity is retained.
#[derive(Clone, Debug)]
pub struct SurrealCrdtStore {
    db: Surreal<Any>,
}

impl SurrealCrdtStore {
    /// Wrap an already-connected and namespace/database-selected `Surreal<Any>`.
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CrdtStore for SurrealCrdtStore {
    #[instrument(skip(self, snapshot), fields(entity_id = %snapshot.entity_id, tenant_id = %snapshot.tenant_id, version = snapshot.version))]
    async fn checkpoint(&self, snapshot: CrdtSnapshot) -> Result<(), PortError> {
        let encoded_hex = hex::encode(&snapshot.encoded);
        let entity_id = snapshot.entity_id.to_string();
        let tenant_id = snapshot.tenant_id.to_string();
        let version = snapshot.version;

        self.db
            .query(
                "UPSERT type::table($table) \
                 SET entity_id = $eid, tenant_id = $tid, \
                     encoded_hex = $hex, version = $ver \
                 WHERE entity_id = $eid AND tenant_id = $tid",
            )
            .bind(("table", TABLE))
            .bind(("eid", entity_id))
            .bind(("tid", tenant_id))
            .bind(("hex", encoded_hex))
            .bind(("ver", version))
            .await
            .map_err(SurrealStoreError::Db)
            .map_err(PortError::from)?;

        Ok(())
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    async fn restore(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<CrdtSnapshot>, PortError> {
        let eid = entity_id.to_string();
        let tid = tenant_id.to_string();

        let mut resp = self
            .db
            .query(
                "SELECT entity_id, tenant_id, encoded_hex, version \
                 FROM type::table($table) \
                 WHERE entity_id = $eid AND tenant_id = $tid \
                 ORDER BY version DESC LIMIT 1",
            )
            .bind(("table", TABLE))
            .bind(("eid", eid))
            .bind(("tid", tid))
            .await
            .map_err(SurrealStoreError::Db)
            .map_err(PortError::from)?;

        let rows: Vec<SnapshotRow> = resp
            .take(0)
            .map_err(|e| SurrealStoreError::Serde(e.to_string()))
            .map_err(PortError::from)?;

        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };

        let encoded_bytes = hex::decode(&row.encoded_hex)
            .map_err(|e| PortError::Transport(format!("hex decode: {e}")))?;

        Ok(Some(CrdtSnapshot {
            entity_id,
            tenant_id,
            encoded: Bytes::from(encoded_bytes),
            version: row.version,
        }))
    }

    #[instrument(skip(self), fields(%entity_id, %tenant_id))]
    async fn purge(&self, entity_id: EntityId, tenant_id: TenantId) -> Result<(), PortError> {
        let eid = entity_id.to_string();
        let tid = tenant_id.to_string();

        self.db
            .query(
                "DELETE type::table($table) \
                 WHERE entity_id = $eid AND tenant_id = $tid",
            )
            .bind(("table", TABLE))
            .bind(("eid", eid))
            .bind(("tid", tid))
            .await
            .map_err(SurrealStoreError::Db)
            .map_err(PortError::from)?;

        Ok(())
    }
}
