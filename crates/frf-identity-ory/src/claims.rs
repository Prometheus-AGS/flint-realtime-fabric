use frf_domain::{SessionId, TenantId};
use frf_ports::VerifiedClaims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::IdentityError;

/// JWT payload claims minted by flint-gate's `claims_enhancement` pre-request hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrfClaims {
    /// Subject — the Kratos identity UUID.
    pub sub: String,
    pub email: Option<String>,
    pub tenant_id: Option<String>,
    pub role: Option<String>,
    pub principal_type: Option<String>,
    pub agent_id: Option<String>,
    pub workflow_id: Option<String>,
    pub scope: Option<String>,
    pub roles: Option<Vec<String>>,
    /// JWT ID. ASO replica tokens require a UUID; legacy non-replica lanes may omit it.
    pub jti: Option<String>,
    pub originating_session_id: Option<String>,
    pub authorization_revision: Option<String>,
    pub projection_revision: Option<u32>,
    pub projection_ids: Option<Vec<String>>,
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

    let carries_replica_contract = claims.scope.as_deref()
        == Some(frf_ports::identity::ASO_REPLICA_SCOPE)
        || claims.originating_session_id.is_some()
        || claims.authorization_revision.is_some()
        || claims.projection_revision.is_some()
        || claims.projection_ids.is_some();
    let session_id = match claims
        .jti
        .as_deref()
        .and_then(|jti| Uuid::parse_str(jti).ok())
    {
        Some(jti) => SessionId::from_uuid(jti),
        None if carries_replica_contract => {
            return Err(IdentityError::MissingClaim(
                "jti (missing or invalid UUID for ASO replica)".to_owned(),
            ));
        }
        None => SessionId::new(),
    };
    let originating_session_id = claims
        .originating_session_id
        .map(|value| {
            Uuid::parse_str(&value)
                .map(SessionId::from_uuid)
                .map_err(|_| {
                    IdentityError::MissingClaim("originating_session_id (invalid UUID)".to_owned())
                })
        })
        .transpose()?;

    Ok(VerifiedClaims {
        session_id,
        originating_session_id,
        tenant_id: TenantId::from_uuid(tenant_uuid),
        subject: claims.sub,
        email: claims.email,
        role: claims.role,
        principal_type: claims.principal_type,
        agent_id: claims.agent_id,
        workflow_id: claims.workflow_id,
        scope: claims.scope,
        roles: claims.roles.unwrap_or_default(),
        authorization_revision: claims.authorization_revision,
        projection_revision: claims.projection_revision,
        projection_ids: claims.projection_ids.unwrap_or_default(),
        expires_at: claims.exp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_ports::identity::{ASO_PROJECTION_REVISION, ASO_REPLICA_SCOPE};
    use serde_json::json;

    fn replica_claims() -> FrfClaims {
        serde_json::from_value(json!({
            "sub": Uuid::from_u128(1).to_string(),
            "tenant_id": Uuid::from_u128(2).to_string(),
            "scope": ASO_REPLICA_SCOPE,
            "jti": Uuid::from_u128(3).to_string(),
            "originating_session_id": Uuid::from_u128(4).to_string(),
            "authorization_revision": "membership:synthetic",
            "projection_revision": ASO_PROJECTION_REVISION,
            "projection_ids": ["cases", "case_evidence", "evidence_states", "evidence_citations", "documents"],
            "aud": "frf-gateway",
            "exp": 9_999_999_999_u64
        }))
        .unwrap()
    }

    #[test]
    fn replica_claims_require_exact_scope_revision_and_session_linkage() {
        let verified = to_verified_claims(replica_claims()).unwrap();
        assert_eq!(
            verified.aso_replica_scope().unwrap(),
            Uuid::from_u128(2).to_string()
        );

        for mutation in ["scope", "revision", "session", "projection"] {
            let mut claims = replica_claims();
            match mutation {
                "scope" => claims.scope = Some("all".to_owned()),
                "revision" => claims.projection_revision = Some(99),
                "session" => claims.originating_session_id = None,
                "projection" => claims.projection_ids = Some(vec!["cases".to_owned()]),
                _ => unreachable!(),
            }
            assert!(
                to_verified_claims(claims)
                    .unwrap()
                    .aso_replica_scope()
                    .is_err()
            );
        }
    }

    #[test]
    fn malformed_token_and_origin_ids_are_not_invented() {
        let mut claims = replica_claims();
        claims.jti = Some("not-a-uuid".to_owned());
        assert!(to_verified_claims(claims).is_err());
        let mut claims = replica_claims();
        claims.jti = None;
        assert!(to_verified_claims(claims).is_err());
        let mut claims = replica_claims();
        claims.originating_session_id = Some("not-a-uuid".to_owned());
        assert!(to_verified_claims(claims).is_err());
    }

    #[test]
    fn non_replica_tokens_keep_legacy_optional_token_ids() {
        for jti in [None, Some("legacy-non-uuid-token-id")] {
            let mut value = json!({
                "sub": Uuid::from_u128(1).to_string(),
                "tenant_id": Uuid::from_u128(2).to_string(),
                "scope": "frf.events.read",
                "aud": "frf-gateway",
                "exp": 9_999_999_999_u64
            });
            if let Some(jti) = jti {
                value["jti"] = json!(jti);
            }
            let claims: FrfClaims = serde_json::from_value(value).unwrap();
            assert!(to_verified_claims(claims).is_ok());
        }
    }
}
