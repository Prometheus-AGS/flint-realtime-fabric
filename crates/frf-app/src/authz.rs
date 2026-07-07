use std::sync::Arc;

use frf_ports::{AuthzProvider, IdentityVerifier, RelationTuple};
use tracing::instrument;

use crate::error::AppError;

/// Request to check, grant, or revoke a relation tuple. The caller's bearer token is
/// verified and its tenant must match `tuple.tenant_id` (defense in depth beneath Keto).
pub struct AuthzRequest {
    pub tuple: RelationTuple,
    pub bearer_token: String,
}

/// Application-layer use-case for the authorization plane (`AuthzService`).
///
/// Wires two ports:
/// - `I: IdentityVerifier` — verifies the bearer JWT and yields the authoritative tenant.
/// - `A: AuthzProvider`    — the Keto-backed relation store (`check`/`write`/`delete`).
///
/// Every operation is tenant-scoped: a caller authenticated for tenant A may not read or
/// mutate a tuple owned by tenant B, even if a stray tuple would otherwise allow it. No
/// adapter crate is imported here; dependency inversion is enforced at the Cargo level.
pub struct AuthzUseCase<A, I> {
    authz: Arc<A>,
    identity: Arc<I>,
}

impl<A, I> AuthzUseCase<A, I>
where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    pub fn new(authz: Arc<A>, identity: Arc<I>) -> Self {
        Self { authz, identity }
    }

    /// Verify the token and enforce tenant-equality against the tuple. Returns the
    /// authoritative tenant on success (unused by callers but keeps the guard explicit).
    async fn authorize(&self, req: &AuthzRequest) -> Result<(), AppError> {
        let claims = self
            .identity
            .verify(&req.bearer_token)
            .await
            .map_err(AppError::Identity)?;

        if claims.tenant_id != req.tuple.tenant_id {
            return Err(AppError::Forbidden(format!(
                "subject {} (tenant {}) may not operate on a relation owned by tenant {}",
                claims.subject, claims.tenant_id, req.tuple.tenant_id
            )));
        }
        Ok(())
    }

    /// Check whether the tuple's subject holds its relation on its object.
    ///
    /// # Errors
    ///
    /// [`AppError::Identity`] on an invalid token; [`AppError::Forbidden`] on a tenant
    /// mismatch; [`AppError::Broker`] on a Keto failure.
    #[instrument(name = "app::authz::check", skip(self, req))]
    pub async fn check(&self, req: AuthzRequest) -> Result<bool, AppError> {
        self.authorize(&req).await?;
        let allowed = self.authz.check(&req.tuple).await?;
        Ok(allowed)
    }

    /// Grant (write) the relation tuple.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check); [`AppError::Broker`] if the Keto write fails.
    #[instrument(name = "app::authz::write", skip(self, req))]
    pub async fn write(&self, req: AuthzRequest) -> Result<(), AppError> {
        self.authorize(&req).await?;
        self.authz.write(req.tuple).await?;
        Ok(())
    }

    /// Revoke (delete) the relation tuple.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check); [`AppError::Broker`] if the Keto delete fails.
    #[instrument(name = "app::authz::delete", skip(self, req))]
    pub async fn delete(&self, req: AuthzRequest) -> Result<(), AppError> {
        self.authorize(&req).await?;
        self.authz.delete(req.tuple).await?;
        Ok(())
    }
}
