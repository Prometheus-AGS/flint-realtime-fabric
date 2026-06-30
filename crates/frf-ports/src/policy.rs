use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::TenantId;
use thiserror::Error;

/// Errors returned by `ActionPolicyProvider`.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("policy evaluation failed: {0}")]
    Evaluation(String),

    #[error("policy store unavailable: {0}")]
    Unavailable(String),
}

/// Cedar-based action-level policy check.
///
/// Cedar governs **mutation actions** (publish, create, update, delete).
/// Visibility is governed by Keto (`AuthzProvider`).  Never conflate the two.
///
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait ActionPolicyProvider: Send + Sync + 'static {
    /// Returns `true` if `principal` is permitted to perform `action` on `resource`.
    ///
    /// `action` is a string such as `"Publish"`, `"Subscribe"`, `"Delete"`.
    /// `resource` is the entity being acted upon (channel ID, entity ID, etc.).
    async fn is_permitted(
        &self,
        principal: &TenantId,
        action: &str,
        resource: &str,
    ) -> Result<bool, PolicyError>;
}

/// No-op `ActionPolicyProvider` — always permits every action.
///
/// Used as the default when `POLICY_ENGINE=none` (the default). Zero overhead.
pub struct NoOpPolicyProvider;

#[async_trait]
impl ActionPolicyProvider for NoOpPolicyProvider {
    async fn is_permitted(
        &self,
        _principal: &TenantId,
        _action: &str,
        _resource: &str,
    ) -> Result<bool, PolicyError> {
        Ok(true)
    }
}

/// Type-erased `ActionPolicyProvider` for use in `AppState` when the concrete
/// policy engine type is chosen at runtime (e.g. from `POLICY_ENGINE` env var).
pub type DynPolicyProvider = Arc<dyn ActionPolicyProvider + Send + Sync>;

/// Sized wrapper around a `DynPolicyProvider`.
///
/// Use as the `P` type parameter in `AppState` when the concrete policy engine
/// is selected at runtime. Delegates all calls through the inner `Arc<dyn ...>`.
#[derive(Clone)]
pub struct BoxedPolicyProvider(pub DynPolicyProvider);

#[async_trait]
impl ActionPolicyProvider for BoxedPolicyProvider {
    async fn is_permitted(
        &self,
        principal: &TenantId,
        action: &str,
        resource: &str,
    ) -> Result<bool, PolicyError> {
        self.0.is_permitted(principal, action, resource).await
    }
}
