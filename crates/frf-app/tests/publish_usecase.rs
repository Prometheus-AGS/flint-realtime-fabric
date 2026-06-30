use std::sync::Arc;

use frf_app::{AppError, PublishRequest, PublishUseCase};
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
use frf_ports::{
    AuthzProvider, EventStream, IdentityVerifier, LogBroker, PortError, RelationTuple,
    VerifiedClaims,
};
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

fn test_envelope() -> EventEnvelope {
    EventEnvelope::new(
        Channel {
            id: ChannelId::new(),
            tenant_id: TenantId::from_uuid(Uuid::nil()),
            path: "test/channel".to_owned(),
        },
        Offset(0),
        EventKind::EntityChange,
        serde_json::Value::Null,
    )
}

fn allow_authz() -> MockAuthz {
    let mut authz = MockAuthz::new();
    authz.expect_check().returning(|_| Ok(true));
    authz
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn returns_offset_on_success() {
    let mut broker = MockBroker::new();
    broker.expect_publish().once().returning(|_| Ok(Offset(42)));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = PublishUseCase::new(
        Arc::new(broker),
        Arc::new(allow_authz()),
        Arc::new(identity),
    );
    let req = PublishRequest {
        envelope: test_envelope(),
        bearer_token: "tok".to_owned(),
    };

    let result = usecase.execute(req).await;
    assert_eq!(result.unwrap(), Offset(42));
}

#[tokio::test]
async fn returns_unauthorized_when_token_invalid() {
    let broker = MockBroker::new();
    let authz = MockAuthz::new();

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Err(PortError::PermissionDenied("bad token".to_owned())));

    let usecase = PublishUseCase::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = PublishRequest {
        envelope: test_envelope(),
        bearer_token: "bad".to_owned(),
    };

    let result = usecase.execute(req).await;
    assert!(
        matches!(result, Err(AppError::Identity(_))),
        "expected Identity error, got {result:?}"
    );
}

#[tokio::test]
async fn returns_forbidden_when_authz_denied() {
    let broker = MockBroker::new();

    let mut authz = MockAuthz::new();
    authz.expect_check().once().returning(|_| Ok(false));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = PublishUseCase::new(Arc::new(broker), Arc::new(authz), Arc::new(identity));
    let req = PublishRequest {
        envelope: test_envelope(),
        bearer_token: "tok".to_owned(),
    };

    let result = usecase.execute(req).await;
    assert!(
        matches!(result, Err(AppError::Forbidden(_))),
        "expected Forbidden error, got {result:?}"
    );
}

#[tokio::test]
async fn propagates_broker_error() {
    let mut broker = MockBroker::new();
    broker
        .expect_publish()
        .once()
        .returning(|_| Err(PortError::Transport("iggy down".to_owned())));

    let mut identity = MockIdentity::new();
    identity
        .expect_verify()
        .once()
        .returning(|_| Ok(test_claims()));

    let usecase = PublishUseCase::new(
        Arc::new(broker),
        Arc::new(allow_authz()),
        Arc::new(identity),
    );
    let req = PublishRequest {
        envelope: test_envelope(),
        bearer_token: "tok".to_owned(),
    };

    let result = usecase.execute(req).await;
    assert!(
        matches!(result, Err(AppError::Broker(_))),
        "expected Broker error, got {result:?}"
    );
}
