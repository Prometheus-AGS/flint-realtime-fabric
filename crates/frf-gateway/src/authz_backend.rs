use async_trait::async_trait;
use frf_authz_keto::KetoAuthzProvider;
use frf_ports::{AuthzProvider, PortError, RelationTuple};

/// Runtime authorization adapter selected by `AUTHZ_BACKEND`.
///
/// The verified-identity mode is intentionally narrow: the gateway verifies the
/// JWT and enforces tenant equality before calling this adapter, while durable
/// application data remains protected by `PostgreSQL` RLS. Keto remains available
/// for platform consumers that require relationship authorization.
pub enum ConfiguredAuthzProvider {
    VerifiedIdentity,
    Keto(KetoAuthzProvider),
}

impl ConfiguredAuthzProvider {
    #[must_use]
    pub fn verified_identity() -> Self {
        Self::VerifiedIdentity
    }

    #[must_use]
    pub fn keto(base_url: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self::Keto(KetoAuthzProvider::new(base_url, namespace))
    }
}

#[async_trait]
impl AuthzProvider for ConfiguredAuthzProvider {
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        match self {
            Self::VerifiedIdentity => Ok(matches!(
                tuple.relation.as_str(),
                "publish" | "subscribe" | "view" | "edit"
            )),
            Self::Keto(provider) => provider.check(tuple).await,
        }
    }

    async fn write(&self, tuple: RelationTuple) -> Result<(), PortError> {
        match self {
            Self::VerifiedIdentity => Err(PortError::PermissionDenied(
                "relationship grants are disabled for verified-identity/RLS authorization"
                    .to_owned(),
            )),
            Self::Keto(provider) => provider.write(tuple).await,
        }
    }

    async fn delete(&self, tuple: RelationTuple) -> Result<(), PortError> {
        match self {
            Self::VerifiedIdentity => Err(PortError::PermissionDenied(
                "relationship revocation is disabled for verified-identity/RLS authorization"
                    .to_owned(),
            )),
            Self::Keto(provider) => provider.delete(tuple).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use frf_domain::{ChannelId, TenantId};
    use frf_ports::{AuthzProvider, RelationTuple};
    use uuid::Uuid;

    use super::ConfiguredAuthzProvider;

    fn tuple(relation: &str) -> RelationTuple {
        RelationTuple {
            tenant_id: TenantId::from_uuid(Uuid::nil()),
            subject: "verified-user".to_owned(),
            relation: relation.to_owned(),
            object: ChannelId::from_uuid(Uuid::nil()).to_string(),
        }
    }

    #[tokio::test]
    async fn verified_identity_allows_non_admin_application_relations() {
        let provider = ConfiguredAuthzProvider::verified_identity();
        for relation in ["publish", "subscribe", "view", "edit"] {
            assert!(provider.check(&tuple(relation)).await.expect("check"));
        }
        assert!(!provider.check(&tuple("user-admin")).await.expect("check"));
    }

    #[tokio::test]
    async fn verified_identity_cannot_mutate_relationships() {
        let provider = ConfiguredAuthzProvider::verified_identity();
        assert!(provider.write(tuple("view")).await.is_err());
        assert!(provider.delete(tuple("view")).await.is_err());
    }
}
