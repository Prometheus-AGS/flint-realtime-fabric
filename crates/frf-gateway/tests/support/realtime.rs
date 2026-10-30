//! Shared fixture for the browser realtime tests: an in-memory broker, a
//! token-table identity verifier, and a gateway router served on a real port.
#![allow(dead_code, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use axum::Router;
use frf_app::SubscribePipeline;
use frf_domain::{
    AgentEvent, Channel, ChannelId, Cursor, EventEnvelope, EventKind, Offset, SessionId,
    SignalEnvelope, TenantId,
};
use frf_gateway::{AppState, GatewayConfig, build_router};
use frf_ports::{
    AgentEventBus, AgentEventStream, AuthzProvider, DynMediaSignaler, EventStream,
    IdentityVerifier, LogBroker, MediaSignaler, NoOpPolicyProvider, PortError, RelationTuple,
    SignalStream, VerifiedClaims,
};
use futures_util::StreamExt as _;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

pub const TENANT: Uuid = Uuid::from_u128(1);
pub const NAMESPACE: Uuid = Uuid::from_u128(0x7ade_9cfb_f939_43de_8a31_ead7_43a7_9f98);

/// Append-only in-memory log with a configurable retention floor.
pub struct MemoryBroker {
    log: Mutex<Vec<EventEnvelope>>,
    live: broadcast::Sender<EventEnvelope>,
    floor: AtomicU64,
}

impl MemoryBroker {
    pub fn new() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
            live: broadcast::channel(256).0,
            floor: AtomicU64::new(0),
        }
    }

    /// Offsets below `floor` are no longer retained.
    pub fn expire_before(&self, floor: u64) {
        self.floor.store(floor, Ordering::SeqCst);
    }

    pub fn published(&self) -> Vec<EventEnvelope> {
        self.log.lock().expect("log").clone()
    }
}

#[async_trait]
impl LogBroker for MemoryBroker {
    async fn publish(&self, mut envelope: EventEnvelope) -> Result<Offset, PortError> {
        let mut log = self.log.lock().expect("log");
        envelope.offset = Offset(log.len() as u64);
        log.push(envelope.clone());
        let _ = self.live.send(envelope.clone());
        Ok(envelope.offset)
    }

    async fn subscribe(
        &self,
        channel_id: ChannelId,
        _consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError> {
        let floor = self.floor.load(Ordering::SeqCst);
        if from != Offset::BEGINNING && from.0 < floor {
            return Err(PortError::NotFound(format!(
                "resnapshot_required: requested broker offset {} precedes retained floor {floor}",
                from.0
            )));
        }
        let live = self.live.subscribe();
        let backlog: Vec<EventEnvelope> = self
            .log
            .lock()
            .expect("log")
            .iter()
            .filter(|e| e.channel.id == channel_id && e.offset >= from && e.offset.0 >= floor)
            .cloned()
            .collect();
        let next = backlog
            .last()
            .map_or(from, |last| last.offset.next())
            .max(from);
        let live = BroadcastStream::new(live).filter_map(move |item| async move {
            item.ok()
                .filter(|e| e.channel.id == channel_id && e.offset >= next)
                .map(Ok)
        });
        Ok(Box::pin(
            futures_util::stream::iter(backlog.into_iter().map(Ok)).chain(live),
        ))
    }

    async fn head_offset(&self, channel_id: ChannelId) -> Result<Option<Offset>, PortError> {
        Ok(self
            .log
            .lock()
            .expect("log")
            .iter()
            .rev()
            .find(|e| e.channel.id == channel_id)
            .map(|e| e.offset))
    }

    async fn seek(&self, _cursor: Cursor) -> Result<(), PortError> {
        Ok(())
    }

    async fn ack(&self, _: ChannelId, _: &str, _: Offset) -> Result<(), PortError> {
        Ok(())
    }

    async fn ensure_channel(&self, _channel: Channel) -> Result<(), PortError> {
        Ok(())
    }
}

/// Every relation tuple is denied, so a passing subscription proves the
/// `topics` claim replaced the tuple check.
pub struct DenyAllAuthz;

#[async_trait]
impl AuthzProvider for DenyAllAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(false)
    }
    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

/// Verifies a token by looking it up; stands in for the JWKS verifier.
#[derive(Default)]
pub struct TokenTable(Mutex<HashMap<String, VerifiedClaims>>);

impl TokenTable {
    pub fn insert(&self, token: &str, claims: VerifiedClaims) {
        self.0
            .lock()
            .expect("tokens")
            .insert(token.to_owned(), claims);
    }
}

#[async_trait]
impl IdentityVerifier for TokenTable {
    async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError> {
        self.0
            .lock()
            .expect("tokens")
            .get(token)
            .cloned()
            .ok_or_else(|| PortError::PermissionDenied("unknown token".to_owned()))
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs()
}

/// Ticket claims: `jti`, `iat = now`, `exp = now + lifetime`, and `topics`.
pub fn ticket_claims(subject: &str, lifetime: u64, topics: &[&str]) -> VerifiedClaims {
    let now = now();
    VerifiedClaims {
        session_id: SessionId::new(),
        originating_session_id: None,
        tenant_id: TenantId::from_uuid(TENANT),
        subject: subject.to_owned(),
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
        expires_at: now + lifetime,
        token_id: Some(Uuid::new_v4().to_string()),
        issued_at: Some(now),
        topics: Some(topics.iter().map(|t| (*t).to_owned()).collect()),
    }
}

pub fn channel_for(topic: &str) -> Channel {
    Channel {
        id: frf_app::topic_channel_id(NAMESPACE, topic),
        tenant_id: TenantId::from_uuid(TENANT),
        path: topic.to_owned(),
    }
}

pub fn envelope(channel: &Channel, n: u64) -> EventEnvelope {
    EventEnvelope::new(
        channel.clone(),
        Offset(0),
        EventKind::Custom("test".to_owned()),
        serde_json::json!({ "n": n }),
    )
}

struct UnusedSignaler;

#[async_trait]
impl MediaSignaler for UnusedSignaler {
    async fn send_signal(&self, _signal: SignalEnvelope) -> Result<(), PortError> {
        Err(PortError::Transport("unused".to_owned()))
    }
    async fn subscribe_signals(
        &self,
        _: SessionId,
        _: TenantId,
    ) -> Result<SignalStream, PortError> {
        Err(PortError::Transport("unused".to_owned()))
    }
    async fn remove_session(&self, _: SessionId, _: TenantId) -> Result<(), PortError> {
        Ok(())
    }
}

struct UnusedAgentBus;

#[async_trait]
impl AgentEventBus for UnusedAgentBus {
    async fn publish(&self, _event: AgentEvent) -> Result<(), PortError> {
        Err(PortError::Transport("unused".to_owned()))
    }
    async fn subscribe(&self, _tenant_id: &str) -> Result<AgentEventStream, PortError> {
        Err(PortError::Transport("unused".to_owned()))
    }
}

pub type Pipeline = SubscribePipeline<MemoryBroker, DenyAllAuthz, TokenTable>;

pub struct Fixture {
    pub broker: Arc<MemoryBroker>,
    pub tokens: Arc<TokenTable>,
    pub pipeline: Arc<Pipeline>,
    pub address: std::net::SocketAddr,
    task: JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn spawn() -> Fixture {
    let broker = Arc::new(MemoryBroker::new());
    let tokens = Arc::new(TokenTable::default());
    let authz = Arc::new(DenyAllAuthz);
    let pipeline = Arc::new(
        SubscribePipeline::new(Arc::clone(&broker), Arc::clone(&authz), Arc::clone(&tokens))
            .with_topic_namespace(Some(NAMESPACE)),
    );
    let router: Router = build_router(Arc::new(AppState {
        subscribe_pipeline: Arc::clone(&pipeline),
        publish_usecase: Arc::new(frf_app::PublishUseCase::new(
            Arc::clone(&broker),
            Arc::clone(&authz),
            Arc::clone(&tokens),
        )),
        media_signaler: Arc::new(DynMediaSignaler::new(Arc::new(UnusedSignaler))),
        agent_bus: Arc::new(UnusedAgentBus),
        identity: Arc::clone(&tokens),
        authz,
        log_broker: Arc::clone(&broker),
        action_policy: Arc::new(NoOpPolicyProvider),
        federation_bridges: vec![],
        media_bridge: None,
        #[cfg(feature = "shape-facade")]
        shape_usecase: None,
        cdc_readiness: tokio::sync::watch::channel(true).1,
        config: Arc::new(GatewayConfig::test_default()),
    }));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    Fixture {
        broker,
        tokens,
        pipeline,
        address,
        task,
    }
}
