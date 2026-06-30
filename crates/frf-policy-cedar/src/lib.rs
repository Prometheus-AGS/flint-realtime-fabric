#![deny(warnings)]
#![warn(clippy::pedantic)]

mod error;

use std::str::FromStr as _;

use async_trait::async_trait;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request,
};
use frf_domain::TenantId;
use frf_ports::{ActionPolicyProvider, PolicyError};
use tracing::instrument;

pub use error::CedarError;

const DEFAULT_POLICY: &str = include_str!("policy.cedar");

/// Cedar-based `ActionPolicyProvider`.
///
/// Wraps an in-memory `PolicySet` loaded from the bundled `policy.cedar` file.
/// For production use, replace the in-memory set with one loaded from a
/// policy store or secret manager.
pub struct CedarPolicyEngine {
    policy_set: PolicySet,
    authorizer: Authorizer,
}

impl CedarPolicyEngine {
    /// Create a new `CedarPolicyEngine` from the default bundled policy.
    ///
    /// # Errors
    ///
    /// Returns an error if the bundled policy is syntactically invalid.
    pub fn new() -> Result<Self, CedarError> {
        Self::from_policy_str(DEFAULT_POLICY)
    }

    /// Create a `CedarPolicyEngine` from a custom policy string.
    ///
    /// # Errors
    ///
    /// Returns an error if the policy string is syntactically invalid.
    pub fn from_policy_str(policy: &str) -> Result<Self, CedarError> {
        let policy_set =
            PolicySet::from_str(policy).map_err(|e| CedarError::PolicyParse(e.to_string()))?;
        Ok(Self {
            policy_set,
            authorizer: Authorizer::new(),
        })
    }
}

#[async_trait]
impl ActionPolicyProvider for CedarPolicyEngine {
    #[instrument(name = "cedar::is_permitted", skip(self), fields(action, resource))]
    async fn is_permitted(
        &self,
        principal: &TenantId,
        action: &str,
        resource: &str,
    ) -> Result<bool, PolicyError> {
        let principal_type = EntityTypeName::from_str("Tenant")
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let principal_id = EntityId::new(principal.to_string());
        let principal_euid = EntityUid::from_type_name_and_id(principal_type, principal_id);

        let action_type = EntityTypeName::from_str("Action")
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let action_id = EntityId::new(action.to_string());
        let action_euid = EntityUid::from_type_name_and_id(action_type, action_id);

        let resource_type = EntityTypeName::from_str("Resource")
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let resource_id = EntityId::new(resource.to_string());
        let resource_euid = EntityUid::from_type_name_and_id(resource_type, resource_id);

        let request = Request::new(
            principal_euid,
            action_euid,
            resource_euid,
            Context::empty(),
            None,
        )
        .map_err(|e| PolicyError::Evaluation(e.to_string()))?;

        let response =
            self.authorizer
                .is_authorized(&request, &self.policy_set, &Entities::empty());

        Ok(response.decision() == Decision::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_domain::TenantId;

    #[tokio::test]
    async fn permits_publish_action() {
        let engine = CedarPolicyEngine::new().expect("valid policy");
        let tenant = TenantId::new();
        let allowed = engine
            .is_permitted(&tenant, "Publish", "channel:test")
            .await
            .expect("evaluation");
        assert!(allowed, "default policy should permit Publish");
    }

    #[tokio::test]
    async fn denies_delete_action_by_default() {
        // The default policy only permits Publish; Delete should be denied.
        let engine = CedarPolicyEngine::new().expect("valid policy");
        let tenant = TenantId::new();
        let allowed = engine
            .is_permitted(&tenant, "Delete", "channel:test")
            .await
            .expect("evaluation");
        assert!(!allowed, "default policy should deny Delete");
    }

    #[tokio::test]
    async fn custom_policy_overrides_default() {
        let policy = r#"permit(principal, action == Action::"Delete", resource);"#;
        let engine = CedarPolicyEngine::from_policy_str(policy).expect("valid policy");
        let tenant = TenantId::new();
        let allowed = engine
            .is_permitted(&tenant, "Delete", "channel:test")
            .await
            .expect("evaluation");
        assert!(allowed, "custom policy should permit Delete");
    }
}
