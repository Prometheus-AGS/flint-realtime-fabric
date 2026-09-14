use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use frf_app::{ShapeCatalog, ShapeRequestTime, ShapeUseCase, ShapeUseCaseError};
use frf_domain::{SessionId, TenantId};
use frf_ports::{
    AuthorizedShapeRequest, AuthzProvider, Cursor, PortError, RelationTuple, ShapeBodyStream,
    ShapeFacade, ShapeHeader, ShapeProtocol, ShapeRequest, ShapeResponse, VerifiedClaims,
};
use futures_util::{Stream, StreamExt as _, stream};

const NOW_EPOCH_SECONDS: u64 = 1_000;

fn request_time(epoch_seconds: u64) -> ShapeRequestTime {
    ShapeRequestTime::new(
        Duration::from_secs(epoch_seconds),
        tokio::time::Instant::now(),
    )
}
const REVALIDATION_INTERVAL: Duration = Duration::from_millis(750);

struct MutableAuthority {
    allowed: AtomicBool,
}

#[async_trait]
impl AuthzProvider for MutableAuthority {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(self.allowed.load(Ordering::Acquire))
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct OneResponseFacade {
    response: Mutex<Option<ShapeResponse>>,
}

#[async_trait]
impl ShapeFacade for OneResponseFacade {
    async fn fetch(&self, _request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError> {
        self.response
            .lock()
            .map_err(|_| PortError::Transport("shape fixture lock poisoned".to_owned()))?
            .take()
            .ok_or_else(|| PortError::Transport("shape fixture already consumed".to_owned()))
    }
}

struct DropObservedStream {
    inner: ShapeBodyStream,
    dropped: Arc<AtomicBool>,
}

impl Stream for DropObservedStream {
    type Item = Result<Bytes, PortError>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(context)
    }
}

impl Drop for DropObservedStream {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

struct Harness {
    use_case: ShapeUseCase<MutableAuthority>,
    authority: Arc<MutableAuthority>,
    upstream_dropped: Arc<AtomicBool>,
    claims: VerifiedClaims,
}

impl Harness {
    fn new(expires_at: u64) -> Self {
        let authority = Arc::new(MutableAuthority {
            allowed: AtomicBool::new(true),
        });
        let upstream_dropped = Arc::new(AtomicBool::new(false));
        let response = ShapeResponse::streamed(
            200,
            vec![
                ShapeHeader::new("electric-handle", b"throttled-handle"),
                ShapeHeader::new("electric-offset", b"10_0"),
            ],
            throttled_body(Arc::clone(&upstream_dropped)),
        );
        let facade = Arc::new(OneResponseFacade {
            response: Mutex::new(Some(response)),
        });
        Self {
            use_case: ShapeUseCase::new(
                catalog(),
                Arc::clone(&authority),
                facade as Arc<dyn ShapeFacade>,
            ),
            authority,
            upstream_dropped,
            claims: claims(expires_at),
        }
    }

    async fn open(&self) -> ShapeResponse {
        match self
            .use_case
            .execute(
                &self.claims,
                initial_request(),
                request_time(NOW_EPOCH_SECONDS),
            )
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("authorized throttled response failed: {error}"),
        }
    }

    async fn assert_handle_released(&self, claims: &VerifiedClaims, now_epoch_seconds: u64) {
        let Err(error) = self
            .use_case
            .execute(
                claims,
                continuation_request(),
                request_time(now_epoch_seconds),
            )
            .await
        else {
            panic!("cancelled response retained its continuation state");
        };
        assert!(matches!(error, ShapeUseCaseError::HandleMismatch));
    }
}

fn throttled_body(dropped: Arc<AtomicBool>) -> ShapeBodyStream {
    let frames = VecDeque::from([
        Bytes::from_static(b"frame-1"),
        Bytes::from_static(b"frame-2"),
        Bytes::from_static(b"frame-3"),
    ]);
    let inner = stream::unfold((frames, true), |(mut frames, first)| async move {
        let frame = frames.pop_front()?;
        if !first {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Some((Ok(frame), (frames, false)))
    });
    Box::pin(DropObservedStream {
        inner: Box::pin(inner),
        dropped,
    })
}

fn catalog() -> ShapeCatalog {
    match ShapeCatalog::from_json(
        r#"{"cases":{"table":"aso.cases","columns":["id","practice_id"],"allowed_params":[],"relation":"view","object_namespace":"practice","scope_column":"practice_id"}}"#,
    ) {
        Ok(catalog) => catalog,
        Err(error) => panic!("invalid test catalog: {error}"),
    }
}

fn claims(expires_at: u64) -> VerifiedClaims {
    VerifiedClaims {
        session_id: SessionId::from_uuid(uuid::Uuid::from_u128(1)),
        originating_session_id: Some(SessionId::from_uuid(uuid::Uuid::from_u128(2))),
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(3)),
        subject: "synthetic-subject".to_owned(),
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
            "cases".to_owned(),
            "case_evidence".to_owned(),
            "documents".to_owned(),
            "evidence_citations".to_owned(),
            "evidence_states".to_owned(),
        ],
        expires_at,
    }
}

fn initial_request() -> ShapeRequest {
    ShapeRequest {
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(3)),
        subject: "ignored-client-subject".to_owned(),
        shape: "cases".to_owned(),
        params: Vec::new(),
        cursor: Cursor::Initial,
        protocol: ShapeProtocol::default(),
    }
}

fn continuation_request() -> ShapeRequest {
    ShapeRequest {
        cursor: Cursor::Resume {
            handle: "throttled-handle".to_owned(),
            offset: "10_0".to_owned(),
        },
        ..initial_request()
    }
}

fn assert_first_frame(frame: Option<Result<Bytes, PortError>>) {
    assert!(matches!(frame, Some(Ok(bytes)) if bytes == Bytes::from_static(b"frame-1")));
}

async fn assert_authority_change_interrupts_body() {
    let harness = Harness::new(NOW_EPOCH_SECONDS + 300);
    let mut response = harness.open().await;
    assert_first_frame(response.body.next().await);

    harness.authority.allowed.store(false, Ordering::Release);
    tokio::time::advance(REVALIDATION_INTERVAL).await;
    tokio::task::yield_now().await;

    assert!(matches!(
        response.body.next().await,
        Some(Err(PortError::PermissionDenied(message)))
            if message == "shape response authority lease ended"
    ));
    assert!(response.body.next().await.is_none());
    harness
        .assert_handle_released(&harness.claims, NOW_EPOCH_SECONDS)
        .await;
    let Err(error) = harness
        .use_case
        .execute(
            &harness.claims,
            initial_request(),
            request_time(NOW_EPOCH_SECONDS),
        )
        .await
    else {
        panic!("new request did not observe ended authority");
    };
    assert!(matches!(error, ShapeUseCaseError::Forbidden));
    assert!(harness.upstream_dropped.load(Ordering::Acquire));
}

#[tokio::test(start_paused = true)]
async fn logout_interrupts_a_nonempty_throttled_body() {
    assert_authority_change_interrupts_body().await;
}

#[tokio::test(start_paused = true)]
async fn membership_change_interrupts_a_nonempty_throttled_body() {
    assert_authority_change_interrupts_body().await;
}

#[tokio::test(start_paused = true)]
async fn grant_expiry_interrupts_a_nonempty_throttled_body() {
    let harness = Harness::new(NOW_EPOCH_SECONDS + 1);
    let mut response = harness.open().await;
    assert_first_frame(response.body.next().await);

    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    assert!(matches!(
        response.body.next().await,
        Some(Err(PortError::PermissionDenied(message)))
            if message == "shape response authority lease ended"
    ));
    assert!(response.body.next().await.is_none());
    harness
        .assert_handle_released(&claims(NOW_EPOCH_SECONDS + 300), NOW_EPOCH_SECONDS + 1)
        .await;
    let Err(error) = harness
        .use_case
        .execute(
            &harness.claims,
            initial_request(),
            request_time(NOW_EPOCH_SECONDS + 1),
        )
        .await
    else {
        panic!("expired grant allowed a new request");
    };
    assert!(matches!(error, ShapeUseCaseError::GrantExpired));
    assert!(harness.upstream_dropped.load(Ordering::Acquire));
}

#[tokio::test(start_paused = true)]
async fn observed_direct_kratos_revocation_drops_the_gate_owned_consumer() {
    let harness = Harness::new(NOW_EPOCH_SECONDS + 300);
    let mut response = harness.open().await;
    assert_first_frame(response.body.next().await);

    harness.authority.allowed.store(false, Ordering::Release);
    drop(response.body);
    tokio::task::yield_now().await;

    assert!(harness.upstream_dropped.load(Ordering::Acquire));
    harness
        .assert_handle_released(&harness.claims, NOW_EPOCH_SECONDS)
        .await;
    let Err(error) = harness
        .use_case
        .execute(
            &harness.claims,
            initial_request(),
            request_time(NOW_EPOCH_SECONDS),
        )
        .await
    else {
        panic!("Gate boundary allowed access after direct revocation");
    };
    assert!(matches!(error, ShapeUseCaseError::Forbidden));
}
