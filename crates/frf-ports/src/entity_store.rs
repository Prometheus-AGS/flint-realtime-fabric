use async_trait::async_trait;
use frf_domain::{EntityChange, EntityId, TenantId};
use futures_core::Stream;

use crate::error::PortError;

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
}
