use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use frf_ports::{ShapeBodyStream, ShapeHeader, ShapeProtocol};
use futures_util::StreamExt as _;

use super::*;

struct StubAuthz {
    allow: bool,
    calls: AtomicUsize,
}

#[async_trait]
impl AuthzProvider for StubAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.allow)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct StubFacade {
    response: ResponseFixture,
    seen: Mutex<Vec<AuthorizedShapeRequest>>,
}

#[derive(Clone)]
struct ResponseFixture {
    status: u16,
    headers: Vec<ShapeHeader>,
    body: Vec<u8>,
}

impl ResponseFixture {
    fn build(&self) -> ShapeResponse {
        ShapeResponse::new(self.status, self.headers.clone(), self.body.clone())
    }
}

struct BlockingFacade {
    called: AtomicBool,
}

#[async_trait]
impl ShapeFacade for BlockingFacade {
    async fn fetch(&self, _request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError> {
        self.called.store(true, Ordering::SeqCst);
        std::future::pending().await
    }
}

struct RevokingAuthz {
    calls: AtomicUsize,
}

#[async_trait]
impl AuthzProvider for RevokingAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(self.calls.fetch_add(1, Ordering::SeqCst) == 0)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct UnavailableAuthz;

#[async_trait]
impl AuthzProvider for UnavailableAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        std::future::pending().await
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

#[async_trait]
impl ShapeFacade for StubFacade {
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError> {
        self.seen.lock().expect("seen lock").push(request.clone());
        Ok(self.response.build())
    }
}

fn claims(subject: &str, token: u128) -> VerifiedClaims {
    VerifiedClaims {
        session_id: SessionId::from_uuid(uuid::Uuid::from_u128(token)),
        originating_session_id: Some(SessionId::from_uuid(uuid::Uuid::from_u128(20))),
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(30)),
        subject: subject.to_owned(),
        email: None,
        role: None,
        principal_type: Some("human".to_owned()),
        agent_id: None,
        workflow_id: None,
        scope: Some(frf_ports::identity::ASO_REPLICA_SCOPE.to_owned()),
        roles: Vec::new(),
        authorization_revision: Some("membership:1".to_owned()),
        projection_revision: Some(frf_ports::identity::ASO_PROJECTION_REVISION),
        projection_ids: vec![
            "case_evidence".to_owned(),
            "cases".to_owned(),
            "documents".to_owned(),
            "evidence_citations".to_owned(),
            "evidence_states".to_owned(),
        ],
        expires_at: 2_000,
    }
}

fn catalog() -> ShapeCatalog {
    ShapeCatalog::from_json(
        r#"{"cases":{"table":"aso.cases","columns":["id","practice_id","gate_affirmed_at"],"allowed_params":[],"relation":"view","object_namespace":"practice","scope_column":"practice_id"}}"#,
    )
    .expect("catalog")
}

fn initial_request() -> ShapeRequest {
    ShapeRequest {
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(30)),
        subject: "ignored-client-subject".to_owned(),
        shape: "cases".to_owned(),
        params: Vec::new(),
        cursor: Cursor::Initial,
        protocol: ShapeProtocol {
            live: Some("true".to_owned()),
            cursor: Some("1234".to_owned()),
            if_none_match: Some("etag-1".to_owned()),
        },
    }
}

fn response(handle: &str) -> ResponseFixture {
    ResponseFixture {
        status: 200,
        headers: vec![ShapeHeader::new("electric-handle", handle.as_bytes())],
        body: b"[]".to_vec(),
    }
}

fn use_case(response: ResponseFixture) -> (ShapeUseCase<StubAuthz>, Arc<StubFacade>) {
    let authz = Arc::new(StubAuthz {
        allow: true,
        calls: AtomicUsize::new(0),
    });
    let facade = Arc::new(StubFacade {
        response,
        seen: Mutex::new(Vec::new()),
    });
    let erased = Arc::clone(&facade) as Arc<dyn ShapeFacade>;
    (ShapeUseCase::new(catalog(), authz, erased), facade)
}

fn epoch(seconds: u64) -> ShapeRequestTime {
    ShapeRequestTime::new(Duration::from_secs(seconds), Instant::now())
}

async fn body_bytes(mut body: ShapeBodyStream) -> Result<Vec<u8>, PortError> {
    let mut bytes = Vec::new();
    while let Some(frame) = body.next().await {
        bytes.extend_from_slice(&frame?);
    }
    Ok(bytes)
}

#[tokio::test]
async fn initial_request_derives_scope_and_preserves_protocol_options() {
    let (use_case, facade) = use_case(response("handle-1"));
    use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(1_000))
        .await
        .expect("authorized response");
    let seen = facade.seen.lock().expect("seen lock");
    let request = seen.first().expect("one request");
    assert_eq!(request.table, "aso.cases");
    assert!(
        request
            .where_clause
            .as_deref()
            .is_some_and(|clause| clause.contains("practice_id ="))
    );
    assert_eq!(request.protocol.live.as_deref(), Some("true"));
    assert_eq!(request.protocol.cursor.as_deref(), Some("1234"));
    assert_eq!(request.protocol.if_none_match.as_deref(), Some("etag-1"));
}

#[tokio::test]
async fn continuation_handle_is_bound_to_the_exact_verified_grant() {
    let (use_case, facade) = use_case(response("handle-1"));
    let owner = claims("subject-1", 10);
    let initial = use_case
        .execute(&owner, initial_request(), epoch(1_000))
        .await
        .expect("initial response");

    let mut continuation = initial_request();
    continuation.cursor = Cursor::Resume {
        handle: "handle-1".to_owned(),
        offset: "10_0".to_owned(),
    };
    let fresh_token_for_same_session = claims("subject-1", 12);
    let provisional_error = use_case
        .execute(
            &fresh_token_for_same_session,
            continuation.clone(),
            epoch(1_001),
        )
        .await
        .expect_err("handle remains provisional until body completion");
    assert!(matches!(
        provisional_error,
        ShapeUseCaseError::HandleMismatch
    ));
    assert_eq!(body_bytes(initial.body).await.expect("initial body"), b"[]");
    use_case
        .execute(
            &fresh_token_for_same_session,
            continuation.clone(),
            epoch(1_001),
        )
        .await
        .expect("owner continuation");

    let error = use_case
        .execute(&claims("subject-2", 11), continuation, epoch(1_001))
        .await
        .expect_err("foreign grant must not reuse handle");
    assert!(matches!(error, ShapeUseCaseError::HandleMismatch));
    assert_eq!(facade.seen.lock().expect("seen lock").len(), 2);
}

#[tokio::test]
async fn dropping_the_body_discards_its_provisional_handle() {
    let (use_case, facade) = use_case(response("provisional-handle"));
    let owner = claims("subject-1", 10);
    let initial = use_case
        .execute(&owner, initial_request(), epoch(1_000))
        .await
        .expect("initial response");
    drop(initial.body);

    let mut continuation = initial_request();
    continuation.cursor = Cursor::Resume {
        handle: "provisional-handle".to_owned(),
        offset: "10_0".to_owned(),
    };
    let error = use_case
        .execute(&owner, continuation, epoch(1_001))
        .await
        .expect_err("dropped response must not publish its handle");

    assert!(matches!(error, ShapeUseCaseError::HandleMismatch));
    assert_eq!(facade.seen.lock().expect("seen lock").len(), 1);
}

#[tokio::test]
async fn shared_upstream_handle_keeps_each_authorized_session_binding() {
    let (use_case, facade) = use_case(response("shared-handle"));
    let first = claims("subject-1", 10);
    let second = claims("subject-2", 11);
    let first_response = use_case
        .execute(&first, initial_request(), epoch(1_000))
        .await
        .expect("first initial response");
    let second_response = use_case
        .execute(&second, initial_request(), epoch(1_000))
        .await
        .expect("second initial response");
    body_bytes(first_response.body)
        .await
        .expect("first initial body");
    body_bytes(second_response.body)
        .await
        .expect("second initial body");

    let mut continuation = initial_request();
    continuation.cursor = Cursor::Resume {
        handle: "shared-handle".to_owned(),
        offset: "10_0".to_owned(),
    };
    use_case
        .execute(&first, continuation.clone(), epoch(1_001))
        .await
        .expect("first session remains bound");
    use_case
        .execute(&second, continuation, epoch(1_001))
        .await
        .expect("second session remains bound");

    assert_eq!(facade.seen.lock().expect("seen lock").len(), 4);
}

#[tokio::test]
async fn expired_handle_is_denied_under_a_fresh_valid_grant() {
    let (use_case, facade) = use_case(response("handle-1"));
    let initial = use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(1_000))
        .await
        .expect("initial response");
    body_bytes(initial.body).await.expect("initial body");

    let mut continuation = initial_request();
    continuation.cursor = Cursor::Resume {
        handle: "handle-1".to_owned(),
        offset: "10_0".to_owned(),
    };
    let mut fresh_grant = claims("subject-1", 12);
    fresh_grant.expires_at = 3_000;
    let error = use_case
        .execute(&fresh_grant, continuation, epoch(2_001))
        .await
        .expect_err("expired handle");
    assert!(matches!(error, ShapeUseCaseError::HandleMismatch));
    assert_eq!(facade.seen.lock().expect("seen lock").len(), 1);
}

#[tokio::test]
async fn expired_grant_is_rejected_before_authorization_or_fetch() {
    let authz = Arc::new(StubAuthz {
        allow: true,
        calls: AtomicUsize::new(0),
    });
    let facade = Arc::new(StubFacade {
        response: response("handle-1"),
        seen: Mutex::new(Vec::new()),
    });
    let use_case = ShapeUseCase::new(
        catalog(),
        Arc::clone(&authz),
        Arc::clone(&facade) as Arc<dyn ShapeFacade>,
    );
    let error = use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(2_000))
        .await
        .expect_err("expired");
    assert!(matches!(error, ShapeUseCaseError::GrantExpired));
    assert_eq!(authz.calls.load(Ordering::SeqCst), 0);
    assert!(facade.seen.lock().expect("seen lock").is_empty());
}

#[tokio::test]
async fn shape_outside_verified_projection_is_rejected_before_data_access() {
    let authz = Arc::new(StubAuthz {
        allow: true,
        calls: AtomicUsize::new(0),
    });
    let facade = Arc::new(StubFacade {
        response: response("handle-1"),
        seen: Mutex::new(Vec::new()),
    });
    let use_case = ShapeUseCase::new(
        catalog(),
        Arc::clone(&authz),
        Arc::clone(&facade) as Arc<dyn ShapeFacade>,
    );
    let verified = claims("subject-1", 10);
    let mut request = initial_request();
    request.shape = "unapproved-projection".to_owned();
    let error = use_case
        .execute(&verified, request, epoch(1_000))
        .await
        .expect_err("projection mismatch");
    assert!(matches!(error, ShapeUseCaseError::Forbidden));
    assert_eq!(authz.calls.load(Ordering::SeqCst), 0);
    assert!(facade.seen.lock().expect("seen lock").is_empty());
}

#[tokio::test]
async fn an_open_response_ends_as_no_content_at_the_active_lease() {
    let authz = Arc::new(StubAuthz {
        allow: true,
        calls: AtomicUsize::new(0),
    });
    let facade = Arc::new(BlockingFacade {
        called: AtomicBool::new(false),
    });
    let use_case = ShapeUseCase::new(
        catalog(),
        authz,
        Arc::clone(&facade) as Arc<dyn ShapeFacade>,
    )
    .with_active_lease(Duration::from_millis(1))
    .expect("bounded lease");

    let response = use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(1_000))
        .await
        .expect("lease expiry is a retryable empty response");

    assert!(facade.called.load(Ordering::SeqCst));
    assert_eq!(response.status, 204);
    assert!(
        body_bytes(response.body)
            .await
            .expect("empty body")
            .is_empty()
    );
    assert!(
        use_case
            .handle_bindings
            .lock()
            .expect("handle bindings lock")
            .is_empty()
    );
}

#[tokio::test]
async fn post_fetch_revocation_discards_rows_and_handle() {
    let authz = Arc::new(RevokingAuthz {
        calls: AtomicUsize::new(0),
    });
    let facade = Arc::new(StubFacade {
        response: response("revoked-handle"),
        seen: Mutex::new(Vec::new()),
    });
    let use_case = ShapeUseCase::new(catalog(), authz, facade as Arc<dyn ShapeFacade>);

    let error = use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(1_000))
        .await
        .expect_err("post-fetch denial must discard protected rows");

    assert!(matches!(error, ShapeUseCaseError::Forbidden));
    assert!(
        use_case
            .handle_bindings
            .lock()
            .expect("handle bindings lock")
            .is_empty()
    );
}

#[tokio::test]
async fn unavailable_authority_cannot_hold_a_response_past_the_lease() {
    let facade = Arc::new(StubFacade {
        response: response("unreachable-handle"),
        seen: Mutex::new(Vec::new()),
    });
    let use_case = ShapeUseCase::new(
        catalog(),
        Arc::new(UnavailableAuthz),
        Arc::clone(&facade) as Arc<dyn ShapeFacade>,
    )
    .with_active_lease(Duration::from_millis(1))
    .expect("bounded lease");

    let response = use_case
        .execute(&claims("subject-1", 10), initial_request(), epoch(1_000))
        .await
        .expect("authority timeout closes without protected rows");

    assert_eq!(response.status, 204);
    assert!(
        body_bytes(response.body)
            .await
            .expect("empty body")
            .is_empty()
    );
    assert!(facade.seen.lock().expect("seen lock").is_empty());
}

#[path = "expiry_regression.rs"]
mod expiry_regression;
