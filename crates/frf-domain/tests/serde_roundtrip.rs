use chrono::Utc;
use frf_domain::{
    agent::{AgentEvent, AgentEventKind, AgentProtocol},
    entity::{ChangeOp, EntityChange},
    envelope::{Channel, Cursor, EventEnvelope, EventKind, Offset},
    ids::{AgentId, ChannelId, EntityId, SessionId, TenantId},
    presence::{Presence, PresenceStatus},
    signal::{SfuMode, SignalEnvelope, SignalKind},
    sync::{SyncOp, SyncOpKind},
};

fn roundtrip<
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
>(
    value: &T,
) {
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(*value, back);
}

fn make_channel() -> Channel {
    Channel {
        id: ChannelId::new(),
        tenant_id: TenantId::new(),
        path: "entity/user/updates".to_string(),
    }
}

#[test]
fn channel_roundtrip() {
    roundtrip(&make_channel());
}

#[test]
fn offset_roundtrip() {
    roundtrip(&Offset(42));
}

#[test]
fn cursor_roundtrip() {
    roundtrip(&Cursor {
        channel_id: ChannelId::new(),
        consumer_id: "worker-1".to_string(),
        offset: Offset(100),
        updated_at: Utc::now(),
    });
}

#[test]
fn event_envelope_roundtrip() {
    let env = EventEnvelope::new(
        make_channel(),
        Offset(1),
        EventKind::EntityChange,
        serde_json::json!({"key": "value"}),
    );
    let json = serde_json::to_string(&env).expect("serialize");
    let back: EventEnvelope = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(env.id, back.id);
    assert_eq!(env.offset, back.offset);
    assert_eq!(env.kind, back.kind);
}

#[test]
fn entity_change_roundtrip() {
    roundtrip(&EntityChange {
        entity_id: EntityId::new(),
        tenant_id: TenantId::new(),
        entity_type: "user".to_string(),
        op: ChangeOp::Update,
        data: serde_json::json!({"name": "Alice"}),
        previous: Some(serde_json::json!({"name": "Bob"})),
        session_id: Some(SessionId::new()),
        timestamp: Utc::now(),
        version: 3,
    });
}

#[test]
fn agent_event_roundtrip() {
    roundtrip(&AgentEvent {
        agent_id: AgentId::new(),
        tenant_id: TenantId::new(),
        session_id: SessionId::new(),
        protocol: AgentProtocol::AgUi,
        kind: AgentEventKind::TextDelta,
        run_id: "run-001".to_string(),
        content: serde_json::json!({"delta": "Hello"}),
        timestamp: Utc::now(),
    });
}

#[test]
fn sync_op_roundtrip() {
    roundtrip(&SyncOp {
        entity_id: EntityId::new(),
        tenant_id: TenantId::new(),
        session_id: SessionId::new(),
        kind: SyncOpKind::Update,
        payload: vec![1, 2, 3, 4],
        lamport: 7,
        timestamp: Utc::now(),
    });
}

#[test]
fn presence_roundtrip() {
    roundtrip(&Presence {
        session_id: SessionId::new(),
        tenant_id: TenantId::new(),
        channel_id: ChannelId::new(),
        user_id: "user-123".to_string(),
        display_name: Some("Alice".to_string()),
        status: PresenceStatus::Online,
        meta: serde_json::json!({"cursor": {"x": 10, "y": 20}}),
        last_seen: Utc::now(),
        expires_at: Utc::now(),
    });
}

#[test]
fn signal_envelope_roundtrip() {
    roundtrip(&SignalEnvelope {
        from_session: SessionId::new(),
        to_session: Some(SessionId::new()),
        tenant_id: TenantId::new(),
        room_id: "room-abc".to_string(),
        kind: SignalKind::Offer,
        sfu_mode: SfuMode::Sovereign,
        payload: serde_json::json!({"sdp": "v=0..."}),
        timestamp: Utc::now(),
    });
}

#[test]
fn ids_roundtrip() {
    use frf_domain::ids::{ChannelId, CursorId};
    roundtrip(&ChannelId::new());
    roundtrip(&CursorId::new());
    roundtrip(&EntityId::new());
    roundtrip(&AgentId::new());
    roundtrip(&SessionId::new());
    roundtrip(&TenantId::new());
}
