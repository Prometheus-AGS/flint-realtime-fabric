use std::sync::Arc;

use frf_app::{AppError, SubscribePipeline, SubscribeRequest};
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
use frf_ports::{
    AuthzProvider, EventStream, IdentityVerifier, LogBroker, PortError, RelationTuple,
    VerifiedClaims,
};
use futures_util::stream;
use mockall::mock;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Mock definitions
// ---------------------------------------------------------------------------

mock! {
    pub Broker {}
    #[async_trait::async_trait]
    impl LogBroker for Broker {
        async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError>;
        async fn subscribe(
            &self,
            channel_id: ChannelId,
            consumer_id: String,
            from: Offset,
        ) -> Result<EventStream, PortError>;
        async fn seek(&self, cursor: frf_domain::Cursor) -> Result<(), PortError>;
        async fn ack(
            &self,
            channel_id: ChannelId,
            consumer_id: &str,
            offset: Offset,
        ) -> Result<(), PortError>;
        async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError>;
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

fn test_claims() -> VerifiedClaims {
    VerifiedClaims {
        session_id: frf_domain::SessionId::new(),
        tenant_id: TenantId::from_uuid(Uuid::nil()),
        subject: "user-123".to_owned(),
        email: None,
        roles: vec![],
    }
}

fn test_envelope(channel_id: ChannelId) -> EventEnvelope {
    EventEnvelope::new(
        Channel {
            id: channel_id,
            tenant_id: TenantId::from_uuid(Uuid::nil()),
            path: "test/channel".to_owned(),
        },
        Offset(0),
        EventKind::EntityChange,
        serde_json::Value::Null,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn returns_stream_when_all_checks_pass() {
    let channel_id = ChannelId::new();
    let envelope = test_envelope(channel_id);

    let mut broker = MockBroker::new();
    let envelope_clone = envelope.clone();
    broker.expect_subscribe().once().returning(move |_, _, _| {
        let ev = envelope_clone.clone();
        let s = stream::once(async move { Ok(ev) });
        Ok(Box::pin(s))
    });

    let mut authz = MockAuthz::new();
    authz.expect_check().returning(|_| Ok(true));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let pipeline = SubscribePipeline::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = SubscribeRequest {
        channel_id,
        bearer_token: "tok".to_owned(),
        from: Offset::BEGINNING,
    };

    let result = pipeline.execute(req).await;
    assert!(result.is_ok(), "expected Ok stream");
}

#[tokio::test]
async fn returns_forbidden_when_subscribe_check_fails() {
    let channel_id = ChannelId::new();

    let broker = MockBroker::new();

    let mut authz = MockAuthz::new();
    authz.expect_check().once().returning(|_| Ok(false));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let pipeline = SubscribePipeline::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = SubscribeRequest {
        channel_id,
        bearer_token: "tok".to_owned(),
        from: Offset::BEGINNING,
    };

    let result = pipeline.execute(req).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "expected Forbidden error"
    );
}

#[tokio::test]
async fn returns_unauthorized_when_token_invalid() {
    let channel_id = ChannelId::new();

    let broker = MockBroker::new();
    let authz = MockAuthz::new();

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Err(PortError::PermissionDenied("bad token".to_owned())));

    let pipeline = SubscribePipeline::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = SubscribeRequest {
        channel_id,
        bearer_token: "bad".to_owned(),
        from: Offset::BEGINNING,
    };

    let result = pipeline.execute(req).await;
    assert!(
        matches!(result, Err(AppError::Identity(_))),
        "expected Identity error"
    );
}

#[tokio::test]
async fn filters_events_where_view_check_fails() {
    use futures_util::StreamExt;

    let channel_id = ChannelId::new();
    let allowed_envelope = test_envelope(channel_id);
    let denied_envelope = test_envelope(channel_id);

    let allowed_id = allowed_envelope.id;
    let denied_id = denied_envelope.id;

    let mut broker = MockBroker::new();
    let allowed_clone = allowed_envelope.clone();
    let denied_clone = denied_envelope.clone();
    broker.expect_subscribe().once().returning(move |_, _, _| {
        let s = stream::iter(vec![Ok(allowed_clone.clone()), Ok(denied_clone.clone())]);
        Ok(Box::pin(s))
    });

    let mut authz = MockAuthz::new();
    authz.expect_check().returning(move |tuple| {
        // subscribe check always passes
        if tuple.relation == "subscribe" {
            return Ok(true);
        }
        // view check: allow only the allowed_id envelope
        let is_allowed = tuple.object == allowed_id.to_string();
        Ok(is_allowed)
    });

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let pipeline = SubscribePipeline::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = SubscribeRequest {
        channel_id,
        bearer_token: "tok".to_owned(),
        from: Offset::BEGINNING,
    };

    let mut stream = pipeline
        .execute(req)
        .await
        .expect("pipeline should succeed");
    let first = stream.next().await;
    let second = stream.next().await;

    assert!(
        matches!(&first, Some(Ok(e)) if e.id == allowed_id),
        "first item should be allowed envelope, got {first:?}"
    );
    assert!(
        second.is_none(),
        "denied envelope should be filtered out, got {second:?}"
    );

    let _ = denied_id; // suppress unused warning
}
