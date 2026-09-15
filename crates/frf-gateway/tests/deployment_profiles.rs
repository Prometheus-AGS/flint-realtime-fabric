#![allow(clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::{Json, Router, routing::get};
use frf_domain::{
    AgentEvent, Channel, ChannelId, Cursor, EventEnvelope, Offset, SessionId, SignalEnvelope,
    TenantId,
};
use frf_gateway::{AppState, AuthzBackend, GatewayConfig, GatewayProfile, build_router};
use frf_ports::{
    AgentEventBus, AgentEventStream, AuthzProvider, DynAgentEventBus, DynMediaSignaler,
    EventStream, IdentityVerifier, LogBroker, MediaSignaler, NoOpPolicyProvider, PortError,
    RelationTuple, SignalStream, VerifiedClaims,
};
use serde_json::json;

struct NoopBroker;

#[async_trait]
impl LogBroker for NoopBroker {
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        Ok(envelope.offset)
    }

    async fn subscribe(
        &self,
        _channel_id: ChannelId,
        _consumer_id: String,
        _from: Offset,
    ) -> Result<EventStream, PortError> {
        Err(PortError::Transport("unused".to_owned()))
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

struct Permit;

#[async_trait]
impl AuthzProvider for Permit {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(true)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct Verify;

#[async_trait]
impl IdentityVerifier for Verify {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Err(PortError::PermissionDenied("unused".to_owned()))
    }
}

struct DisabledMedia;

#[async_trait]
impl MediaSignaler for DisabledMedia {
    async fn send_signal(&self, _signal: SignalEnvelope) -> Result<(), PortError> {
        Err(PortError::PermissionDenied("disabled".to_owned()))
    }

    async fn subscribe_signals(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        Err(PortError::PermissionDenied("disabled".to_owned()))
    }

    async fn remove_session(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        Ok(())
    }
}

struct DisabledAgents;

#[async_trait]
impl AgentEventBus for DisabledAgents {
    async fn publish(&self, _event: AgentEvent) -> Result<(), PortError> {
        Err(PortError::PermissionDenied("disabled".to_owned()))
    }

    async fn subscribe(&self, _tenant_id: &str) -> Result<AgentEventStream, PortError> {
        Err(PortError::PermissionDenied("disabled".to_owned()))
    }
}

type TestState =
    AppState<NoopBroker, Permit, Verify, DynMediaSignaler, DynAgentEventBus, NoOpPolicyProvider>;

fn state(config: GatewayConfig) -> Arc<TestState> {
    state_with_cdc_readiness(config, true)
}

fn state_with_cdc_readiness(config: GatewayConfig, cdc_ready: bool) -> Arc<TestState> {
    let broker = Arc::new(NoopBroker);
    let authz = Arc::new(Permit);
    let identity = Arc::new(Verify);
    Arc::new(AppState {
        subscribe_pipeline: Arc::new(frf_app::SubscribePipeline::new(
            Arc::clone(&broker),
            Arc::clone(&authz),
            Arc::clone(&identity),
        )),
        publish_usecase: Arc::new(frf_app::PublishUseCase::new(
            Arc::clone(&broker),
            Arc::clone(&authz),
            Arc::clone(&identity),
        )),
        media_signaler: Arc::new(DynMediaSignaler::new(Arc::new(DisabledMedia))),
        agent_bus: Arc::new(DynAgentEventBus::new(Arc::new(DisabledAgents))),
        identity,
        authz,
        log_broker: broker,
        action_policy: Arc::new(NoOpPolicyProvider),
        federation_bridges: Vec::new(),
        media_bridge: None,
        #[cfg(feature = "shape-facade")]
        shape_usecase: None,
        cdc_readiness: tokio::sync::watch::channel(cdc_ready).1,
        config: Arc::new(config),
    })
}

fn enable_cdc(config: &mut GatewayConfig) {
    config.cdc_enabled = true;
    config.cdc_replication_url = Some("postgres://postgres:5432/frf".to_owned());
    config.cdc_slot_name = Some("frf_slot".to_owned());
    config.cdc_publication_name = Some("frf_pub".to_owned());
    config.cdc_tenant_id = Some(uuid::Uuid::nil());
    config.cdc_channel_path = Some("entities".to_owned());
}

async fn dependency_http() -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new()
        .route(
            "/jwks",
            get(|| async { Json(json!({"keys": [{"kty": "RSA", "kid": "acceptance"}]})) }),
        )
        .route("/health/ready", get(|| async { "ready" }))
        .route("/v1/health", get(|| async { "ready" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind dependency fixture");
    let address = listener.local_addr().expect("dependency address");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve dependency fixture");
    });
    (format!("http://{address}"), task)
}

async fn iggy_tcp() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Iggy handshake fixture");
    let address = listener.local_addr().expect("Iggy fixture address");
    let task = tokio::spawn(async move { while listener.accept().await.is_ok() {} });
    (format!("iggy://user:secret@{address}"), task)
}

#[tokio::test]
async fn full_profile_reaches_semantic_readiness() {
    let (dependency, dependency_task) = dependency_http().await;
    let (iggy, iggy_task) = iggy_tcp().await;
    let mut config = GatewayConfig::test_default();
    config.profile = GatewayProfile::Full;
    config.iggy_connection_string = iggy;
    config.authz_backend = AuthzBackend::Keto;
    config.keto_base_url = dependency.clone();
    config.gateway_jwks_url = format!("{dependency}/jwks");
    enable_cdc(&mut config);
    config.validate().expect("valid full profile");

    let server = axum_test::TestServer::new(build_router(state(config))).expect("test server");
    let response = server.get("/readyz").await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert!(
        body["checks"]["authorization"]["ok"]
            .as_bool()
            .unwrap_or(false)
    );
    assert!(body["checks"]["jwks"]["ok"].as_bool().unwrap_or(false));
    assert!(body["checks"]["iggy"]["ok"].as_bool().unwrap_or(false));
    assert!(body["checks"]["cdc"]["ok"].as_bool().unwrap_or(false));
    dependency_task.abort();
    iggy_task.abort();
}

#[cfg(feature = "shape-facade")]
#[tokio::test]
async fn shape_only_profile_reaches_semantic_readiness_without_iggy() {
    let (dependency, dependency_task) = dependency_http().await;
    let mut config = GatewayConfig::test_default();
    config.profile = GatewayProfile::ShapeOnly;
    config.grpc_port = None;
    config.iggy_connection_string.clear();
    config.gateway_jwks_url = format!("{dependency}/jwks");
    config.shape_electric_url = Some(dependency);
    config.shape_catalog_path = Some("/run/config/shape-catalog.json".to_owned());
    config.validate().expect("valid shape-only profile");

    let server = axum_test::TestServer::new(build_router(state(config))).expect("test server");
    let response = server.get("/readyz").await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert!(body["checks"]["electric"]["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        body["checks"]["iggy"]["detail"],
        "disabled: outside shape-only profile"
    );
    dependency_task.abort();
}

#[tokio::test]
async fn full_profile_readiness_fails_when_authority_is_unreachable() {
    let (dependency, dependency_task) = dependency_http().await;
    let (iggy, iggy_task) = iggy_tcp().await;
    let mut config = GatewayConfig::test_default();
    config.iggy_connection_string = iggy;
    config.authz_backend = AuthzBackend::Keto;
    config.keto_base_url = "http://127.0.0.1:1".to_owned();
    config.gateway_jwks_url = format!("{dependency}/jwks");

    let server = axum_test::TestServer::new(build_router(state(config))).expect("test server");
    server
        .get("/readyz")
        .await
        .assert_status_service_unavailable();
    dependency_task.abort();
    iggy_task.abort();
}

#[tokio::test]
async fn full_profile_readiness_fails_when_cdc_stream_is_inactive() {
    let (dependency, dependency_task) = dependency_http().await;
    let (iggy, iggy_task) = iggy_tcp().await;
    let mut config = GatewayConfig::test_default();
    config.iggy_connection_string = iggy;
    config.authz_backend = AuthzBackend::Keto;
    config.keto_base_url = dependency.clone();
    config.gateway_jwks_url = format!("{dependency}/jwks");
    enable_cdc(&mut config);
    config.validate().expect("valid full profile");

    let server = axum_test::TestServer::new(build_router(state_with_cdc_readiness(config, false)))
        .expect("test server");
    let response = server.get("/readyz").await;
    response.assert_status_service_unavailable();
    assert_eq!(
        response.json::<serde_json::Value>()["checks"]["cdc"]["detail"],
        "logical replication stream inactive"
    );
    dependency_task.abort();
    iggy_task.abort();
}
