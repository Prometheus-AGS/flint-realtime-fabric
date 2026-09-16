use async_trait::async_trait;
use frf_domain::{EntityChange, EntityId, TenantId};
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

    /// Install a current-state snapshot and its inclusive WAL/broker boundary.
    /// Returns false when an equal or newer projection already exists.
    async fn install_projection_snapshot(
        &self,
        snapshot: EntityProjectionSnapshot,
    ) -> Result<bool, PortError>;

    /// Return the last state-and-cursor position committed by this projection.
    async fn projection_checkpoint(&self) -> Result<Option<ProjectionCursor>, PortError>;
}
