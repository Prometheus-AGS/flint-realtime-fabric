#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Router;
use axum::routing::get;
use axum_test::TestServer;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset, SessionId, TenantId};
use frf_gateway::{AppState, GatewayConfig};
use frf_ports::{
    AuthorizedShapeRequest, AuthzProvider, IdentityVerifier, LogBroker, PortError, RelationTuple,
    ShapeChunk, ShapeFacade, VerifiedClaims,
};
use frf_shape_electric::{ShapeCatalog, ShapeResolver};

struct NoopBroker;

#[async_trait]
impl LogBroker for NoopBroker {
    async fn publish(&self, _envelope: EventEnvelope) -> Result<Offset, PortError> {
        Ok(Offset::BEGINNING)
    }

    async fn subscribe(
        &self,
        _channel_id: ChannelId,
        _consumer_id: String,
        _from: Offset,
    ) -> Result<frf_ports::EventStream, PortError> {
        Ok(Box::pin(tokio_stream::empty()))
    }

    async fn seek(&self, _cursor: Cursor) -> Result<(), PortError> {
        Ok(())
    }

    async fn ack(
        &self,
        _channel_id: ChannelId,
        _consumer_id: &str,
        _offset: Offset,
    ) -> Result<(), PortError> {
        Ok(())
    }

    async fn ensure_channel(&self, _channel: Channel) -> Result<(), PortError> {
        Ok(())
    }
}

#[derive(Default)]
struct CountingAuthz {
    calls: AtomicUsize,
}

#[async_trait]
impl AuthzProvider for CountingAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct FixedIdentity {
    claims: VerifiedClaims,
}

#[async_trait]
impl IdentityVerifier for FixedIdentity {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(self.claims.clone())
    }
}

#[derive(Default)]
struct CountingFacade {
    calls: AtomicUsize,
    body: Vec<u8>,
    seen: Mutex<Option<AuthorizedShapeRequest>>,
}

#[async_trait]
impl ShapeFacade for CountingFacade {
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeChunk, PortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.seen.lock().expect("facade request lock") = Some(request.clone());
        Ok(ShapeChunk::new(
            "unused".to_owned(),
            "unused".to_owned(),
            self.body.clone(),
            false,
            true,
        ))
    }
}

fn claims(scope: &str, projection_revision: u32) -> VerifiedClaims {
    VerifiedClaims {
        session_id: SessionId::new(),
        originating_session_id: Some(SessionId::new()),
        tenant_id: TenantId::new(),
        subject: "test-subject".to_owned(),
        email: None,
        role: None,
        principal_type: Some("human".to_owned()),
        agent_id: None,
        workflow_id: None,
        scope: Some(scope.to_owned()),
        roles: Vec::new(),
        authorization_revision: Some("membership-revision".to_owned()),
        projection_revision: Some(projection_revision),
        projection_ids: vec![
            "case_evidence".to_owned(),
            "cases".to_owned(),
            "documents".to_owned(),
            "evidence_citations".to_owned(),
            "evidence_states".to_owned(),
        ],
        expires_at: 9_999_999_999,
    }
}

fn mounted_server(
    claims: VerifiedClaims,
    catalog: ShapeCatalog,
    body: Vec<u8>,
) -> (TestServer, Arc<CountingAuthz>, Arc<CountingFacade>) {
    let broker = Arc::new(NoopBroker);
    let authz = Arc::new(CountingAuthz::default());
    let identity = Arc::new(FixedIdentity { claims });
    let facade = Arc::new(CountingFacade {
        calls: AtomicUsize::new(0),
        body,
        seen: Mutex::new(None),
    });
    let resolver = Arc::new(ShapeResolver::new(catalog, authz.clone(), "practice_id"));

    let state = Arc::new(AppState {
        subscribe_pipeline: Arc::new(frf_app::SubscribePipeline::new(
            broker.clone(),
            authz.clone(),
            identity.clone(),
        )),
        publish_usecase: Arc::new(frf_app::PublishUseCase::new(
            broker.clone(),
            authz.clone(),
            identity.clone(),
        )),
        media_signaler: Arc::new(()),
        agent_bus: Arc::new(()),
        identity,
        authz: authz.clone(),
        log_broker: broker,
        action_policy: Arc::new(()),
        federation_bridges: Vec::new(),
        media_bridge: None,
        shape_facade: Some(facade.clone()),
        shape_resolver: Some(resolver),
        config: Arc::new(GatewayConfig::test_default()),
    });

    let app = Router::new()
        .route(
            "/v1/shape",
            get(frf_gateway::routes::shape::get_shape::<
                NoopBroker,
                CountingAuthz,
                FixedIdentity,
                (),
                (),
                (),
            >),
        )
        .with_state(state);
    (TestServer::new(app).expect("test server"), authz, facade)
}

async fn assert_grant_rejected_before_data_access(claims: VerifiedClaims) {
    let (server, authz, facade) = mounted_server(claims, ShapeCatalog::default(), Vec::new());

    let response = server
        .get("/v1/shape?shape=cases")
        .add_header("authorization", "Bearer projection-token")
        .await;

    assert_eq!(response.status_code(), 403);
    assert_eq!(authz.calls.load(Ordering::SeqCst), 0);
    assert_eq!(facade.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mounted_shape_route_rejects_wrong_scope_before_resolver_or_facade() {
    assert_grant_rejected_before_data_access(claims("another.service.read", 1)).await;
}

#[tokio::test]
async fn mounted_shape_route_rejects_wrong_revision_before_resolver_or_facade() {
    assert_grant_rejected_before_data_access(claims("aso.replica.read", 2)).await;
}

#[tokio::test]
async fn authorized_shape_route_preserves_a_cleared_gate_summary() {
    let verified = claims("aso.replica.read", 1);
    let practice_id = verified.tenant_id.to_string();
    let body = br#"[{"id":"synthetic-case","gate_affirmed_at":null}]"#.to_vec();
    let catalog = ShapeCatalog::from_json(
        r#"{
            "cases": {
                "table": "aso.cases",
                "columns": [
                    "id",
                    "practice_id",
                    "status",
                    "gate_affirmed_at",
                    "created_at",
                    "updated_at"
                ],
                "allowed_params": [],
                "relation": "view",
                "object_namespace": "practice"
            }
        }"#,
    )
    .expect("cases shape policy");
    let (server, authz, facade) = mounted_server(verified, catalog, body.clone());

    let response = server
        .get("/v1/shape?shape=cases")
        .add_header("authorization", "Bearer projection-token")
        .await;

    response.assert_status_ok();
    assert_eq!(response.as_bytes(), body.as_slice());
    assert_eq!(authz.calls.load(Ordering::SeqCst), 1);
    assert_eq!(facade.calls.load(Ordering::SeqCst), 1);
    let seen = facade.seen.lock().expect("facade request lock");
    let request = seen.as_ref().expect("authorized request");
    assert_eq!(request.table, "aso.cases");
    assert_eq!(
        request.columns,
        [
            "id",
            "practice_id",
            "status",
            "gate_affirmed_at",
            "created_at",
            "updated_at",
        ]
    );
    assert!(
        !request
            .columns
            .iter()
            .any(|column| column == "gate_affirmed_by")
    );
    assert_eq!(
        request.where_clause,
        format!("practice_id = '{practice_id}'")
    );
}
