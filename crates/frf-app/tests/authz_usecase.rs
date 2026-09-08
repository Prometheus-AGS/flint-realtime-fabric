#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

use std::sync::Arc;

use frf_app::{AppError, AuthzRequest, AuthzUseCase};
use frf_domain::TenantId;
use frf_ports::{AuthzProvider, IdentityVerifier, PortError, RelationTuple, VerifiedClaims};
use mockall::mock;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Mock definitions
// ---------------------------------------------------------------------------

mock! {
    pub Authz {}
    #[async_trait::async_trait]
    impl AuthzProvider for Authz {
        async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError>;
        async fn write(&self, tuple: RelationTuple) -> Result<(), PortError>;
        async fn delete(&self, tuple: RelationTuple) -> Result<(), PortError>;
    }
}

mock! {
    pub Identity {}
    #[async_trait::async_trait]
    impl IdentityVerifier for Identity {
        async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError>;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const NIL_TENANT: Uuid = Uuid::nil();

fn test_claims() -> VerifiedClaims {
    VerifiedClaims {
        session_id: frf_domain::SessionId::new(),
        originating_session_id: None,
        tenant_id: TenantId::from_uuid(NIL_TENANT),
        subject: "admin-1".to_owned(),
        email: None,
        role: None,
        principal_type: None,
        agent_id: None,
        workflow_id: None,
        scope: None,
        roles: vec![],
        authorization_revision: None,
        projection_revision: None,
        projection_ids: vec![],
        expires_at: 9_999_999_999,
    }
}

fn tuple_for(tenant: TenantId) -> RelationTuple {
    RelationTuple {
        tenant_id: tenant,
        subject: "user-9".to_owned(),
        relation: "view".to_owned(),
        object: "entity-42".to_owned(),
    }
}

fn req(tenant: TenantId) -> AuthzRequest {
    AuthzRequest {
        tuple: tuple_for(tenant),
        bearer_token: "tok".to_owned(),
    }
}

fn expect_verified(identity: &mut MockIdentity) {
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn check_returns_allowed_true() {
    let mut authz = MockAuthz::new();
    authz.expect_check().once().returning(|_| Ok(true));
    let mut identity = MockIdentity::new();
    expect_verified(&mut identity);

    let usecase = AuthzUseCase::new(Arc::new(authz), Arc::new(identity));
    let allowed = usecase
        .check(req(TenantId::from_uuid(NIL_TENANT)))
        .await
        .unwrap();
    assert!(allowed);
}

#[tokio::test]
async fn check_returns_allowed_false() {
    let mut authz = MockAuthz::new();
    authz.expect_check().once().returning(|_| Ok(false));
    let mut identity = MockIdentity::new();
    expect_verified(&mut identity);

    let usecase = AuthzUseCase::new(Arc::new(authz), Arc::new(identity));
    let allowed = usecase
        .check(req(TenantId::from_uuid(NIL_TENANT)))
        .await
        .unwrap();
    assert!(!allowed);
}

#[tokio::test]
async fn write_then_delete_roundtrip() {
    let mut authz = MockAuthz::new();
    authz.expect_write().once().returning(|_| Ok(()));
    authz.expect_delete().once().returning(|_| Ok(()));
    let authz = Arc::new(authz);

    let mut id_w = MockIdentity::new();
    expect_verified(&mut id_w);
    let uc_w = AuthzUseCase::new(Arc::clone(&authz), Arc::new(id_w));
    uc_w.write(req(TenantId::from_uuid(NIL_TENANT)))
        .await
        .unwrap();

    let mut id_d = MockIdentity::new();
    expect_verified(&mut id_d);
    let uc_d = AuthzUseCase::new(authz, Arc::new(id_d));
    uc_d.delete(req(TenantId::from_uuid(NIL_TENANT)))
        .await
        .unwrap();
}

#[tokio::test]
async fn rejects_foreign_tenant_before_provider() {
    // Claims are for the nil tenant; the tuple targets a different tenant. The guard
    // must reject BEFORE the provider is consulted.
    let authz = MockAuthz::new(); // no expectations → panics if reached
    let mut identity = MockIdentity::new();
    expect_verified(&mut identity);

    let usecase = AuthzUseCase::new(Arc::new(authz), Arc::new(identity));
    let foreign = TenantId::from_uuid(Uuid::from_u128(0xdead_beef));
    let result = usecase.write(req(foreign)).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "expected Forbidden (tenant mismatch), got {result:?}"
    );
}

#[tokio::test]
async fn propagates_provider_error() {
    let mut authz = MockAuthz::new();
    authz
        .expect_check()
        .once()
        .returning(|_| Err(PortError::Transport("keto down".to_owned())));
    let mut identity = MockIdentity::new();
    expect_verified(&mut identity);

    let usecase = AuthzUseCase::new(Arc::new(authz), Arc::new(identity));
    let result = usecase.check(req(TenantId::from_uuid(NIL_TENANT))).await;
    assert!(
        matches!(result, Err(AppError::Broker(_))),
        "expected Broker error, got {result:?}"
    );
}

#[tokio::test]
async fn rejects_invalid_token() {
    let authz = MockAuthz::new();
    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Err(PortError::PermissionDenied("bad token".to_owned())));

    let usecase = AuthzUseCase::new(Arc::new(authz), Arc::new(identity));
    let result = usecase.check(req(TenantId::from_uuid(NIL_TENANT))).await;
    assert!(
        matches!(result, Err(AppError::Identity(_))),
        "expected Identity error, got {result:?}"
    );
}
