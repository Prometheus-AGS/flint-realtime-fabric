#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

use std::sync::Arc;

use frf_app::{AppError, EntityRequest, EntityUseCase};
use frf_domain::{ChangeOp, EntityChange, EntityId, TenantId};
use frf_ports::{
    AuthzProvider, EntityChangeStream, EntityStore, IdentityVerifier, PortError, RelationTuple,
    VerifiedClaims,
};
use mockall::mock;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Mock definitions
// ---------------------------------------------------------------------------

mock! {
    pub Store {}
    #[async_trait::async_trait]
    impl EntityStore for Store {
        async fn get_entity(
            &self,
            entity_id: EntityId,
            tenant_id: TenantId,
        ) -> Result<Option<EntityChange>, PortError>;
        async fn watch_entity(
            &self,
            entity_id: EntityId,
            tenant_id: TenantId,
        ) -> Result<EntityChangeStream, PortError>;
    }
}

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
        tenant_id: TenantId::from_uuid(NIL_TENANT),
        subject: "user-123".to_owned(),
        email: None,
        role: None,
        principal_type: None,
        agent_id: None,
        workflow_id: None,
        scope: None,
        roles: vec![],
    }
}

fn sample_change(entity: EntityId, tenant: TenantId) -> EntityChange {
    EntityChange {
        entity_id: entity,
        tenant_id: tenant,
        entity_type: "widget".to_owned(),
        op: ChangeOp::Upsert,
        data: serde_json::json!({ "hello": "world" }),
        previous: None,
        session_id: None,
        timestamp: chrono::Utc::now(),
        version: 1,
    }
}

fn allow_authz() -> MockAuthz {
    let mut authz = MockAuthz::new();
    authz.expect_check().returning(|_| Ok(true));
    authz
}

fn req(entity: EntityId, tenant: TenantId) -> EntityRequest {
    EntityRequest {
        entity_id: entity,
        tenant_id: tenant,
        bearer_token: "tok".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_returns_entity_when_authorized() {
    let entity = EntityId::new();
    let tenant = TenantId::from_uuid(NIL_TENANT);

    let mut store = MockStore::new();
    store
        .expect_get_entity()
        .once()
        .returning(move |_, _| Ok(Some(sample_change(entity, tenant))));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = EntityUseCase::new(Arc::new(store), Arc::new(allow_authz()), Arc::new(identity));
    let got = usecase.get(req(entity, tenant)).await.unwrap();
    assert_eq!(got.unwrap().entity_id, entity);
}

#[tokio::test]
async fn get_rejects_foreign_tenant_before_store() {
    // Claims are for the nil tenant; the request targets a different tenant. The
    // tenant-equality guard must reject BEFORE authz or the store are consulted.
    let store = MockStore::new(); // no expect_get_entity → panics if reached
    let authz = MockAuthz::new(); // no expect_check → panics if reached

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = EntityUseCase::new(Arc::new(store), Arc::new(authz), Arc::new(identity));
    let foreign = TenantId::from_uuid(Uuid::from_u128(0xdead_beef));
    let result = usecase.get(req(EntityId::new(), foreign)).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "expected Forbidden (tenant mismatch), got {result:?}"
    );
}

#[tokio::test]
async fn get_returns_forbidden_when_view_denied() {
    let store = MockStore::new(); // store must not be reached when view is denied

    let mut authz = MockAuthz::new();
    authz.expect_check().once().returning(|_| Ok(false));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = EntityUseCase::new(Arc::new(store), Arc::new(authz), Arc::new(identity));
    let tenant = TenantId::from_uuid(NIL_TENANT);
    let result = usecase.get(req(EntityId::new(), tenant)).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "expected Forbidden (view denied), got {result:?}"
    );
}

#[tokio::test]
async fn get_returns_unauthenticated_when_token_invalid() {
    let store = MockStore::new();
    let authz = MockAuthz::new();

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Err(PortError::PermissionDenied("bad token".to_owned())));

    let usecase = EntityUseCase::new(Arc::new(store), Arc::new(authz), Arc::new(identity));
    let tenant = TenantId::from_uuid(NIL_TENANT);
    let result = usecase.get(req(EntityId::new(), tenant)).await;
    assert!(
        matches!(result, Err(AppError::Identity(_))),
        "expected Identity error, got {result:?}"
    );
}

#[tokio::test]
async fn get_returns_none_for_unknown_entity() {
    let mut store = MockStore::new();
    store.expect_get_entity().once().returning(|_, _| Ok(None));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = EntityUseCase::new(Arc::new(store), Arc::new(allow_authz()), Arc::new(identity));
    let tenant = TenantId::from_uuid(NIL_TENANT);
    let got = usecase.get(req(EntityId::new(), tenant)).await.unwrap();
    assert!(got.is_none());
}
