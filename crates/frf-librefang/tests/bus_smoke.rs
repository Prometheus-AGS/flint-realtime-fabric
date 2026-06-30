use frf_domain::{
    AgentEvent, AgentEventKind, AgentProtocol,
    ids::{AgentId, SessionId, TenantId},
};
use frf_librefang::LibreFangBus;
use frf_ports::AgentEventBus;
use std::time::Duration;
use tokio_stream::StreamExt;

fn make_event(tenant_id: TenantId) -> AgentEvent {
    AgentEvent {
        agent_id: AgentId::new(),
        tenant_id,
        session_id: SessionId::new(),
        protocol: AgentProtocol::AgUi,
        kind: AgentEventKind::TextDelta,
        run_id: "run-001".to_owned(),
        content: serde_json::json!({"type": "text_delta", "delta": "hello"}),
        timestamp: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn publish_delivers_to_subscriber() {
    let bus = LibreFangBus::start().expect("bus should start");

    let tenant = TenantId::new();
    let tenant_str = tenant.as_uuid().to_string();

    let mut stream = bus
        .subscribe(&tenant_str)
        .await
        .expect("subscribe should succeed");

    let event = make_event(tenant);

    bus.publish(event.clone())
        .await
        .expect("publish should succeed");

    let received = tokio::time::timeout(Duration::from_millis(500), stream.next())
        .await
        .expect("should receive within 500ms")
        .expect("stream should yield an event");

    assert_eq!(received.run_id, event.run_id);
    assert_eq!(received.kind, AgentEventKind::TextDelta);
}

#[tokio::test]
async fn event_for_other_tenant_not_delivered() {
    let bus = LibreFangBus::start().expect("bus should start");

    let tenant_a = TenantId::new();
    let tenant_b = TenantId::new();

    let mut stream_a = bus
        .subscribe(&tenant_a.as_uuid().to_string())
        .await
        .expect("subscribe should succeed");

    let event_b = make_event(tenant_b);
    bus.publish(event_b).await.expect("publish should succeed");

    let result = tokio::time::timeout(Duration::from_millis(100), stream_a.next()).await;

    assert!(
        result.is_err(),
        "tenant A should not receive tenant B's event"
    );
}
