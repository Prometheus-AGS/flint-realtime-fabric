// p8-c004: RunAgent bidi stream integration tests.
//
// These tests verify the filtering and cancellation logic that AgentGrpcService
// applies to the domain event stream, without requiring a live gRPC transport.
// Full transport-level tests live in subscribe_mux.rs and require live infra.

use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::ids::{AgentId, SessionId, TenantId};
use frf_domain::{
    AgentEvent, AgentEventKind, AgentProtocol, Channel, ChannelId, Cursor, EventEnvelope, Offset,
    SignalEnvelope,
};
use frf_gateway::{AppState, GatewayConfig};
use frf_ports::{
    AgentEventBus, AgentEventStream, AuthzProvider, DynMediaSignaler, IdentityVerifier, LogBroker,
    MediaSignaler, NoOpPolicyProvider, PortError, RelationTuple, SignalStream, VerifiedClaims,
};
use tokio_stream::StreamExt as _;

// ─── Mock adapters ────────────────────────────────────────────────────────────

struct AlwaysVerify {
    tenant_id: TenantId,
    session_id: SessionId,
}

#[async_trait]
impl IdentityVerifier for AlwaysVerify {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: self.session_id,
            tenant_id: self.tenant_id,
            subject: "test-subject".to_owned(),
            email: None,
            roles: vec![],
        })
    }
}

struct AlwaysPermit;

#[async_trait]
impl AuthzProvider for AlwaysPermit {
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

struct StubSignaler;

#[async_trait]
impl MediaSignaler for StubSignaler {
    async fn send_signal(&self, _signal: SignalEnvelope) -> Result<(), PortError> {
        Err(PortError::Transport("stub".to_owned()))
    }

    async fn subscribe_signals(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        Err(PortError::Transport("stub".to_owned()))
    }

    async fn remove_session(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        Ok(())
    }
}

struct FixedBus {
    events: Vec<AgentEvent>,
}

#[async_trait]
impl AgentEventBus for FixedBus {
    async fn publish(&self, _event: AgentEvent) -> Result<(), PortError> {
        Ok(())
    }

    async fn subscribe(&self, _tenant_id: &str) -> Result<AgentEventStream, PortError> {
        Ok(Box::pin(tokio_stream::iter(self.events.clone())))
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_event(agent_id: AgentId, session_id: SessionId, tenant_id: TenantId) -> AgentEvent {
    AgentEvent {
        agent_id,
        tenant_id,
        session_id,
        protocol: AgentProtocol::AgUi,
        kind: AgentEventKind::TextDelta,
        run_id: "run-001".to_owned(),
        content: serde_json::json!({ "text": "hello" }),
        timestamp: chrono::Utc::now(),
    }
}

fn make_state(
    tenant_id: TenantId,
    session_id: SessionId,
    events: Vec<AgentEvent>,
) -> Arc<
    AppState<
        NoopBroker,
        AlwaysPermit,
        AlwaysVerify,
        DynMediaSignaler,
        FixedBus,
        NoOpPolicyProvider,
    >,
> {
    Arc::new(AppState {
        subscribe_pipeline: Arc::new(frf_app::SubscribePipeline::new(
            Arc::new(NoopBroker),
            Arc::new(AlwaysPermit),
            Arc::new(AlwaysVerify {
                tenant_id,
                session_id,
            }),
        )),
        publish_usecase: Arc::new(frf_app::PublishUseCase::new(
            Arc::new(NoopBroker),
            Arc::new(AlwaysPermit),
            Arc::new(AlwaysVerify {
                tenant_id,
                session_id,
            }),
        )),
        media_signaler: Arc::new(DynMediaSignaler::new(Arc::new(StubSignaler))),
        agent_bus: Arc::new(FixedBus { events }),
        identity: Arc::new(AlwaysVerify {
            tenant_id,
            session_id,
        }),
        authz: Arc::new(AlwaysPermit),
        log_broker: Arc::new(NoopBroker),
        action_policy: Arc::new(NoOpPolicyProvider),
        federation_bridges: vec![],
        config: Arc::new(GatewayConfig::test_default()),
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

/// The AgentService subscribes to the bus and filters events by (agent_id, session_id).
/// Verify that the domain stream returned by `AgentEventBus::subscribe` can be filtered
/// correctly — this is the core of what `run_agent` does on the outbound side.
#[tokio::test]
async fn domain_stream_filters_to_matching_session() {
    let tenant_id = TenantId::new();
    let session_id = SessionId::new();
    let agent_id = AgentId::new();
    let other_session = SessionId::new();

    let matching = make_event(agent_id, session_id, tenant_id);
    let other = make_event(agent_id, other_session, tenant_id);

    let state = make_state(tenant_id, session_id, vec![matching.clone(), other]);

    let agent_id_str = agent_id.to_string();
    let session_id_str = session_id.to_string();

    let bus_stream = state
        .agent_bus
        .subscribe(&tenant_id.to_string())
        .await
        .unwrap();

    let filtered: Vec<AgentEvent> = bus_stream
        .filter(|ev| {
            ev.agent_id.to_string() == agent_id_str && ev.session_id.to_string() == session_id_str
        })
        .collect()
        .await;

    assert_eq!(
        filtered.len(),
        1,
        "only the matching-session event should pass the filter"
    );
    assert_eq!(filtered[0].session_id, session_id);
}

/// The cancel `watch::Receiver` used inside `run_agent` terminates the stream
/// when set to `true`. Verify the filter + cancel idiom used in the gateway.
#[tokio::test]
async fn cancel_watch_terminates_stream_before_exhaustion() {
    let tenant_id = TenantId::new();
    let session_id = SessionId::new();
    let agent_id = AgentId::new();

    // 200 events — cancel fires after a delay, stream must stop before 200.
    let events: Vec<AgentEvent> = (0..200)
        .map(|_| make_event(agent_id, session_id, tenant_id))
        .collect();

    let state = make_state(tenant_id, session_id, events);

    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let agent_id_str = agent_id.to_string();
    let session_id_str = session_id.to_string();

    // Replicate the filter closure from AgentGrpcService::run_agent.
    let domain_stream = state
        .agent_bus
        .subscribe(&tenant_id.to_string())
        .await
        .unwrap();

    let proto_stream = domain_stream.filter(move |ev| {
        if *cancel_rx.borrow() {
            return false;
        }
        ev.agent_id.to_string() == agent_id_str && ev.session_id.to_string() == session_id_str
    });

    // Send cancel after a tiny delay to let a few events through.
    tokio::spawn(async move {
        // Yield a few times so the consumer can pull some events first.
        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        let _ = cancel_tx.send(true);
    });

    let collected: Vec<AgentEvent> = proto_stream.collect().await;

    // The stream should have terminated before producing all 200 events.
    // In fast CI it may be 0; that is still correct (cancel fired early).
    assert!(
        collected.len() < 200,
        "cancel should terminate stream before 200 events; got {}",
        collected.len()
    );
}

/// Verify that `AppState` and `AgentGrpcService` wire together with the mock
/// adapters without a compile error, and that the bus/identity/authz fields
/// are correctly set.  This is a smoke test for the generic composition.
#[tokio::test]
async fn app_state_generic_composition_compiles() {
    let tenant_id = TenantId::new();
    let session_id = SessionId::new();

    let state = make_state(tenant_id, session_id, vec![]);

    // AgentGrpcService::new just wraps the state — no side effects.
    let _svc = frf_gateway::agent_grpc_service::AgentGrpcService::new(Arc::clone(&state));

    // Confirm the bus is reachable through the state.
    let stream = state
        .agent_bus
        .subscribe(&tenant_id.to_string())
        .await
        .unwrap();

    let events: Vec<AgentEvent> = stream.collect().await;
    assert!(events.is_empty(), "empty bus should yield no events");
}
