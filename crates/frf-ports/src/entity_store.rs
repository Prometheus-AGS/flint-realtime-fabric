use async_trait::async_trait;
use frf_domain::{EntityChange, EntityId, EntityTypeDelivery, EntityTypeSelector, TenantId};
use futures_core::Stream;

use crate::error::PortError;

/// Durable source and broker position committed with one projected mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionCursor {
    pub source_epoch: String,
    pub commit_lsn: u64,
    pub transaction_index: u32,
    pub broker_offset: u64,
}

/// Current entity rows captured at a source boundary before WAL catch-up.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityProjectionSnapshot {
    pub entities: Vec<EntityChange>,
    pub cursor: ProjectionCursor,
}

/// One durable current row with the lossless typed source representation used
/// by the v2 entity-type watch surface.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedEntityProjection {
    pub change: EntityChange,
    pub delivery: EntityTypeDelivery,
}

/// Current typed rows and the inclusive projection boundary captured atomically.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTypeProjectionSnapshot {
    pub entities: Vec<EntityTypeDelivery>,
    pub cursor: Option<ProjectionCursor>,
}

/// Result of applying a broker mutation to the durable projection.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionApply {
    /// State and cursor advanced atomically; carries the effective watch event.
    Applied(Box<EntityChange>),
    /// The position was already included by a snapshot or earlier replay.
    Duplicate,
}

/// A stream of entity changes from an `EntityStore` watch subscription.
/// Boxed to keep the trait object-safe.
pub type EntityChangeStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<EntityChange, PortError>> + Send>>;

/// Read and watch domain entities by id within a tenant.
///
/// The read side of the entity plane (`EntityService`): `get` returns the latest
/// materialized `EntityChange` for an entity; `watch` streams subsequent changes.
/// Tenant scoping is a required argument — an implementation MUST NOT return an entity
/// belonging to a different tenant than the one requested.
///
/// Adapter crates MUST instrument their implementations with
/// `#[tracing::instrument(name = "port::EntityStore::<method>")]`.
#[async_trait]
pub trait EntityStore: Send + Sync + 'static {
    /// Fetch the latest known change for an entity within a tenant.
    /// Returns `Ok(None)` when the entity is unknown to this tenant.
    async fn get_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<Option<EntityChange>, PortError>;

    /// Stream subsequent changes for an entity within a tenant. The stream ends when the
    /// subscription is dropped; each item is one `EntityChange`.
    async fn watch_entity(
        &self,
        entity_id: EntityId,
        tenant_id: TenantId,
    ) -> Result<EntityChangeStream, PortError>;

    /// Atomically apply one committed mutation and advance its durable cursor.
    async fn apply_projection(
        &self,
        change: EntityChange,
        cursor: ProjectionCursor,
    ) -> Result<ProjectionApply, PortError>;

    /// Atomically apply a mutation to both the v1 projection and the lossless
    /// typed representation used by `WatchEntityType`.
    async fn apply_typed_projection(
        &self,
        projection: TypedEntityProjection,
        cursor: ProjectionCursor,
    ) -> Result<ProjectionApply, PortError> {
        self.apply_projection(projection.change, cursor).await
    }

    /// Capture current rows for one approved type and the inclusive cursor in
    /// the same store boundary. Adapters without typed projection support fail
    /// closed rather than synthesizing a partial snapshot.
    async fn snapshot_entity_type(
        &self,
        _entity_type: &EntityTypeSelector,
        _tenant_id: TenantId,
    ) -> Result<EntityTypeProjectionSnapshot, PortError> {
        Err(PortError::NotFound(
            "resnapshot_required: typed entity projection is unavailable".to_owned(),
        ))
    }

    /// Install a current-state snapshot and its inclusive WAL/broker boundary.
    /// Returns false when an equal or newer projection already exists.
    async fn install_projection_snapshot(
        &self,
        snapshot: EntityProjectionSnapshot,
    ) -> Result<bool, PortError>;

    /// Return the last state-and-cursor position committed by this projection.
    async fn projection_checkpoint(&self) -> Result<Option<ProjectionCursor>, PortError>;
}
