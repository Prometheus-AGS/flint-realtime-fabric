use async_trait::async_trait;
use frf_domain::{SessionId, TenantId};

use crate::error::PortError;

/// Claims extracted from a verified JWT.
#[derive(Debug, Clone)]
pub struct VerifiedClaims {
    pub session_id: SessionId,
    pub tenant_id: TenantId,
    pub subject: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

/// JWT / OIDC token verification at the gateway boundary.
///
/// Implemented by `frf-identity-ory` (Kratos + Oathkeeper). Never trust
/// unverified claims downstream — call this once per connection.
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait IdentityVerifier: Send + Sync + 'static {
    /// Verify a raw JWT bearer token. Returns extracted claims on success.
    async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError>;
}
