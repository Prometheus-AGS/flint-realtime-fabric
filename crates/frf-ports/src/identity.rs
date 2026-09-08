use async_trait::async_trait;
use frf_domain::{SessionId, TenantId};

use crate::error::PortError;

/// Claims extracted from a verified JWT.
#[derive(Debug, Clone)]
pub struct VerifiedClaims {
    /// Unique identifier for this token instance (`jti`).
    pub session_id: SessionId,
    /// Kratos session that authorized the ASO replica token.
    pub originating_session_id: Option<SessionId>,
    pub tenant_id: TenantId,
    pub subject: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub principal_type: Option<String>,
    pub agent_id: Option<String>,
    pub workflow_id: Option<String>,
    pub scope: Option<String>,
    pub roles: Vec<String>,
    pub authorization_revision: Option<String>,
    pub projection_revision: Option<u32>,
    pub projection_ids: Vec<String>,
    pub expires_at: u64,
}

pub const ASO_REPLICA_SCOPE: &str = "aso.replica.read";
pub const ASO_PROJECTION_REVISION: u32 = 1;

impl VerifiedClaims {
    /// Validate the dedicated ASO replica claim contract and return its
    /// server-derived practice scope.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::PermissionDenied`] when any required grant claim is
    /// absent or differs from the versioned ASO contract.
    pub fn aso_replica_scope(&self) -> Result<String, PortError> {
        if self.scope.as_deref() != Some(ASO_REPLICA_SCOPE) {
            return Err(PortError::PermissionDenied("ASO replica scope".to_owned()));
        }
        if self.projection_revision != Some(ASO_PROJECTION_REVISION) {
            return Err(PortError::PermissionDenied(
                "ASO projection revision".to_owned(),
            ));
        }
        if self.originating_session_id.is_none()
            || self
                .authorization_revision
                .as_deref()
                .is_none_or(str::is_empty)
        {
            return Err(PortError::PermissionDenied(
                "ASO session linkage".to_owned(),
            ));
        }
        let expected = [
            "case_evidence",
            "cases",
            "documents",
            "evidence_citations",
            "evidence_states",
        ];
        let mut actual = self
            .projection_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        actual.sort_unstable();
        if actual != expected {
            return Err(PortError::PermissionDenied(
                "ASO projection allowlist".to_owned(),
            ));
        }
        Ok(self.tenant_id.to_string())
    }
}

/// JWT / OIDC token verification at the gateway boundary.
///
/// Implemented by `frf-identity-ory` (Kratos + flint-gate). Never trust
/// unverified claims downstream — call this once per connection.
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait IdentityVerifier: Send + Sync + 'static {
    /// Verify a raw JWT bearer token. Returns extracted claims on success.
    async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError>;
}
