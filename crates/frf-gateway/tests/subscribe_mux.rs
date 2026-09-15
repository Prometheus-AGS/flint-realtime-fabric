#![allow(clippy::expect_used)] // integration test: failures must retain actionable context

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::{Json, Router, routing::get};
use frf_broker_iggy::IggyBroker;
use frf_domain::{
    AgentEvent, Channel, ChannelId, Cursor, EventEnvelope, EventKind, Offset, SessionId,
    SignalEnvelope, TenantId,
};
use frf_gateway::authz_backend::ConfiguredAuthzProvider;
use frf_gateway::{AppState, GatewayConfig, build_router};
use frf_identity_ory::OryIdentityVerifier;
use frf_ports::{
    AgentEventBus, AgentEventStream, DynMediaSignaler, EventStream, LogBroker, MediaSignaler,
    NoOpPolicyProvider, PortError, SignalStream,
};
use futures_util::StreamExt as _;
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest as _};

const IO_TIMEOUT: Duration = Duration::from_secs(15);
const TENANT_UUID: uuid::Uuid = uuid::Uuid::from_u128(1);

struct UnusedSignaler;

#[async_trait]
impl MediaSignaler for UnusedSignaler {
    async fn send_signal(&self, _signal: SignalEnvelope) -> Result<(), PortError> {
        Err(PortError::Transport(
            "media is outside this fixture".to_owned(),
        ))
    }

    async fn subscribe_signals(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        Err(PortError::Transport(
            "media is outside this fixture".to_owned(),
        ))
    }

    async fn remove_session(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        Ok(())
    }
}

struct UnusedAgentBus;

#[async_trait]
impl AgentEventBus for UnusedAgentBus {
    async fn publish(&self, _event: AgentEvent) -> Result<(), PortError> {
        Err(PortError::Transport(
            "agent events are outside this fixture".to_owned(),
        ))
    }

    async fn subscribe(&self, _tenant_id: &str) -> Result<AgentEventStream, PortError> {
        Err(PortError::Transport(
            "agent events are outside this fixture".to_owned(),
        ))
    }
}

struct DeliveryToggleBroker {
    inner: IggyBroker,
    disabled: bool,
}

#[async_trait]
impl LogBroker for DeliveryToggleBroker {
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        self.inner.publish(envelope).await
    }

    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError> {
        let stream = self.inner.subscribe(channel_id, consumer_id, from).await?;
        if self.disabled {
            Ok(Box::pin(stream.filter_map(|_| async {
                None::<Result<EventEnvelope, PortError>>
            })))
        } else {
            Ok(stream)
        }
    }

    async fn seek(&self, cursor: Cursor) -> Result<(), PortError> {
        self.inner.seek(cursor).await
    }

    async fn ack(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
        offset: Offset,
    ) -> Result<(), PortError> {
        self.inner.ack(channel_id, consumer_id, offset).await
    }

    async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError> {
        self.inner.ensure_channel(channel).await
    }
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("required integration input {name} is missing"))
}

async fn spawn_jwks(jwks: Value) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind JWKS fixture");
    let address = listener.local_addr().expect("read JWKS fixture address");
    let app = Router::new().route(
        "/jwks.json",
        get(move || {
            let body = jwks.clone();
            async move { Json(body) }
        }),
    );
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve JWKS fixture");
    });
    (format!("http://{address}/jwks.json"), task)
}

async fn integration_router(iggy_url: &str, jwks_url: String, channel: &Channel) -> Router {
    let inner = tokio::time::timeout(IO_TIMEOUT, IggyBroker::new(iggy_url))
        .await
        .expect("Iggy connection deadline exceeded")
        .expect("connect to owned Iggy fixture");
    let broker = Arc::new(DeliveryToggleBroker {
        inner,
        disabled: std::env::var("FRF_DISABLE_DELIVERY").as_deref() == Ok("1"),
    });
    tokio::time::timeout(IO_TIMEOUT, broker.ensure_channel(channel.clone()))
        .await
        .expect("Iggy channel creation deadline exceeded")
        .expect("create integration channel");

    let authz = Arc::new(ConfiguredAuthzProvider::verified_identity());
    let identity = Arc::new(OryIdentityVerifier::with_issuer(
        jwks_url,
        "frf-gateway",
        "frf-e2e",
    ));
    build_router(Arc::new(AppState {
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
        media_signaler: Arc::new(DynMediaSignaler::new(Arc::new(UnusedSignaler))),
        agent_bus: Arc::new(UnusedAgentBus),
        identity,
        authz,
        log_broker: broker,
        action_policy: Arc::new(NoOpPolicyProvider),
        federation_bridges: vec![],
        media_bridge: None,
        config: Arc::new(GatewayConfig::test_default()),
    }))
}

async fn spawn_gateway(router: Router) -> (std::net::SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind gateway fixture");
    let address = listener.local_addr().expect("read gateway fixture address");
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve gateway fixture");
    });
    (address, task)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the owned compose.integration.yml fixture; run scripts/run-local-integration.sh"]
async fn subscriber_receives_published_event() {
    let iggy_url = required_env("FRF_TEST_IGGY_URL");
    let token = required_env("FRF_TEST_JWT");
    let jwks_file = required_env("FRF_TEST_JWKS_FILE");
    let jwks: Value = serde_json::from_slice(
        &std::fs::read(jwks_file).expect("read disposable integration JWKS"),
    )
    .expect("parse disposable integration JWKS");
    let (jwks_url, jwks_task) = spawn_jwks(jwks).await;

    let channel = Channel {
        id: ChannelId::new(),
        tenant_id: TenantId::from_uuid(TENANT_UUID),
        path: "integration/authenticated-publish".to_owned(),
    };
    let router = integration_router(&iggy_url, jwks_url, &channel).await;
    let (address, gateway_task) = spawn_gateway(router).await;

    let mut request = format!("ws://{address}/ws/v1/subscribe?channel={}", channel.id)
        .into_client_request()
        .expect("build WebSocket request");
    request.headers_mut().insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {token}")
            .parse()
            .expect("build Authorization header"),
    );
    let (mut socket, response) = tokio::time::timeout(IO_TIMEOUT, connect_async(request))
        .await
        .expect("WebSocket connection deadline exceeded")
        .expect("open authenticated WebSocket subscription");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

    let expected_payload = json!({"fixture": "pri-c002", "authenticated": true});
    let envelope = EventEnvelope::new(
        channel.clone(),
        Offset(1),
        EventKind::EntityChange,
        expected_payload.clone(),
    );
    let response = tokio::time::timeout(
        IO_TIMEOUT,
        reqwest::Client::new()
            .post(format!("http://{address}/v1/publish"))
            .bearer_auth(&token)
            .json(&envelope)
            .send(),
    )
    .await
    .expect("HTTP publish deadline exceeded")
    .expect("send authenticated HTTP publish");
    assert_eq!(response.status(), StatusCode::OK);

    let delivery_timeout = if std::env::var("FRF_DISABLE_DELIVERY").as_deref() == Ok("1") {
        Duration::from_secs(2)
    } else {
        IO_TIMEOUT
    };
    let frame = tokio::time::timeout(delivery_timeout, socket.next())
        .await
        .expect("required delivery missing: WebSocket receive deadline exceeded")
        .expect("required delivery missing: WebSocket stream ended")
        .expect("required delivery missing: WebSocket transport failed");
    let received: EventEnvelope = serde_json::from_str(
        frame
            .to_text()
            .expect("required delivery was not a text frame"),
    )
    .expect("required delivery was not an EventEnvelope");
    assert_eq!(received.id, envelope.id);
    assert_eq!(received.channel, channel);
    assert_eq!(received.payload, expected_payload);
    println!("required delivery received: {}", received.id);

    gateway_task.abort();
    jwks_task.abort();
}
