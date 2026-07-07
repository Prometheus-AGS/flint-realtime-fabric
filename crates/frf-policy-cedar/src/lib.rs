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

/// Cedar-based `ActionPolicyProvider` governing mutation actions.
///
/// Wraps an in-memory `PolicySet` loaded from the bundled `policy.cedar` file.
/// For production use, replace the in-memory set with one loaded from a policy
/// store or secret manager.
///
/// # Scope and limitations
///
/// This engine evaluates **action-level** policies — `permit`/`forbid` keyed on
/// `action == Action::"…"` with no entity-attribute conditions. Requests are
/// evaluated against an **empty entity store** (`Entities::empty()`). A policy
/// that references principal or resource *attributes* has no entities to resolve
/// them against; Cedar reports that as an authorization error, which
/// [`CedarPolicyEngine::is_permitted`] logs and propagates as
/// [`PolicyError::Evaluation`] — it does **not** silently deny. Attribute-based
/// ABAC requires an entity store and is out of scope until one exists.
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
        let action_id = EntityId::new(action);
        let action_euid = EntityUid::from_type_name_and_id(action_type, action_id);

        let resource_type = EntityTypeName::from_str("Resource")
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let resource_id = EntityId::new(resource);
        let resource_euid = EntityUid::from_type_name_and_id(resource_type, resource_id);

        let request = Request::new(
            principal_euid,
            action_euid,
            resource_euid,
            Context::empty(),
            None,
        )
        .map_err(|e| PolicyError::Evaluation(e.to_string()))?;

        // NOTE: entities are intentionally empty. This engine evaluates
        // ACTION-level policies (e.g. `permit(principal, action == Action::"Publish",
        // resource)`) which need no entity attributes. A policy that references
        // principal/resource ATTRIBUTES cannot match without an entity store, and
        // Cedar surfaces that as an authorization *error* (not a silent deny) — we
        // detect and propagate those below so such a policy fails loudly rather than
        // silently denying every request.
        let response =
            self.authorizer
                .is_authorized(&request, &self.policy_set, &Entities::empty());

        // Surface policy evaluation errors instead of collapsing them into a silent
        // deny. A broken or attribute-requiring policy is an operator misconfiguration
        // that must be visible, not a quiet denial.
        let errors: Vec<String> = response
            .diagnostics()
            .errors()
            .map(ToString::to_string)
            .collect();
        if !errors.is_empty() {
            let joined = errors.join("; ");
            tracing::error!(action, resource, errors = %joined, "Cedar policy evaluation error");
            return Err(PolicyError::Evaluation(format!(
                "Cedar policy evaluation error(s): {joined}"
            )));
        }

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

    #[tokio::test]
    async fn attribute_requiring_policy_surfaces_error_not_silent_deny() {
        // A policy that references a principal attribute cannot be evaluated
        // against the empty entity store. Cedar reports an authorization error,
        // which must propagate as Err — NOT collapse into a silent deny.
        let policy = r#"permit(principal, action == Action::"Publish", resource) when { principal.role == "admin" };"#;
        let engine = CedarPolicyEngine::from_policy_str(policy).expect("valid policy");
        let tenant = TenantId::new();

        let result = engine
            .is_permitted(&tenant, "Publish", "channel:test")
            .await;

        assert!(
            matches!(result, Err(PolicyError::Evaluation(_))),
            "attribute-requiring policy must surface an evaluation error, got {result:?}"
        );
    }
}
