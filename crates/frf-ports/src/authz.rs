use async_trait::async_trait;
use frf_domain::TenantId;

use crate::error::PortError;

/// Zanzibar-style relation tuple: subject has `relation` on object.
#[derive(Debug, Clone)]
pub struct RelationTuple {
    pub tenant_id: TenantId,
    pub subject: String,
    pub relation: String,
    pub object: String,
}

/// `ReBAC` / Zanzibar permission check.
///
/// Implemented by `frf-authz-keto`. Every fan-out delivery calls `check`
/// before emitting to the subscriber — cache at subscribe-time to amortize
/// per-event Keto latency.
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait AuthzProvider: Send + Sync + 'static {
    /// Return `true` if `subject` has `relation` on `object` within tenant.
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError>;

    /// Write a relation tuple (grant).
    async fn write(&self, tuple: RelationTuple) -> Result<(), PortError>;

    /// Delete a relation tuple (revoke).
    async fn delete(&self, tuple: RelationTuple) -> Result<(), PortError>;
}
