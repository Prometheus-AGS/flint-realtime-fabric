use frf_domain::{SessionId, TenantId};
use frf_ports::VerifiedClaims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::IdentityError;

/// JWT payload claims emitted by Oathkeeper's `id_token` mutator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrfClaims {
    /// Subject — the Kratos identity UUID.
    pub sub: String,
    pub email: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Option<Vec<String>>,
    /// JWT ID — used as the session identifier.
    pub jti: Option<String>,
    /// Audience — must contain the expected value.
    pub aud: serde_json::Value,
    /// Expiry timestamp (Unix seconds). Validated by `jsonwebtoken`.
    pub exp: u64,
}

/// Convert raw JWT claims into the port-level `VerifiedClaims`.
///
/// # Errors
///
/// Returns [`IdentityError::MissingClaim`] when a required field is absent or invalid.
pub fn to_verified_claims(claims: FrfClaims) -> Result<VerifiedClaims, IdentityError> {
    let tenant_id_str = claims
        .tenant_id
        .ok_or_else(|| IdentityError::MissingClaim("tenant_id".to_owned()))?;

    let tenant_uuid = Uuid::parse_str(&tenant_id_str)
        .map_err(|_| IdentityError::MissingClaim("tenant_id (invalid UUID)".to_owned()))?;

    let session_id = if let Some(jti) = &claims.jti {
        let jti_uuid = Uuid::parse_str(jti).unwrap_or_else(|_| Uuid::new_v4());
        SessionId::from_uuid(jti_uuid)
    } else {
        SessionId::new()
    };

    Ok(VerifiedClaims {
        session_id,
        tenant_id: TenantId::from_uuid(tenant_uuid),
        subject: claims.sub,
        email: claims.email,
        roles: claims.roles.unwrap_or_default(),
    })
}
