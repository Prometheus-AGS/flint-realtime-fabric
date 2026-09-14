use std::collections::VecDeque;
use std::future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use frf_app::{ShapeCatalog, ShapeRequestTime, ShapeUseCase, ShapeUseCaseError};
use frf_domain::{SessionId, TenantId};
use frf_ports::{
    AuthorizedShapeRequest, AuthzProvider, Cursor, PortError, RelationTuple, ShapeBodyStream,
    ShapeFacade, ShapeHeader, ShapeProtocol, ShapeRequest, ShapeResponse, VerifiedClaims,
};
use futures_util::StreamExt as _;

#[path = "support/shape_timing.rs"]
mod shape_timing;

use shape_timing::{EventKind, Timeline, timed_body};

const NOW_EPOCH_SECONDS: u64 = 1_000;

fn request_time(epoch_seconds: u64) -> ShapeRequestTime {
    ShapeRequestTime::new(
        Duration::from_secs(epoch_seconds),
        tokio::time::Instant::now(),
    )
}
const TEST_LEASE: Duration = Duration::from_secs(1);
const MAX_TEST_LEASE: Duration = Duration::from_millis(1_750);
const REVALIDATION_INTERVAL: Duration = Duration::from_millis(750);
const AUTHORITY_TIMEOUT: Duration = Duration::from_millis(750);

#[derive(Clone, Copy)]
enum PostOpenAuthority {
    Allow,
    Unavailable,
    Pending,
}

struct ScriptedAuthority {
    calls: AtomicUsize,
    post_open: PostOpenAuthority,
    timeline: Arc<Timeline>,
}

#[async_trait]
impl AuthzProvider for ScriptedAuthority {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        let call = self.calls.fetch_add(1, Ordering::AcqRel);
        self.timeline.record(EventKind::AuthorityCheck(call));
        if call < 2 {
            return Ok(true);
        }
        match self.post_open {
            PostOpenAuthority::Allow => Ok(true),
            PostOpenAuthority::Unavailable => {
                Err(PortError::Transport("authority unavailable".to_owned()))
            }
            PostOpenAuthority::Pending => future::pending().await,
        }
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct QueueFacade {
    responses: Mutex<VecDeque<ShapeResponse>>,
}

#[async_trait]
impl ShapeFacade for QueueFacade {
    async fn fetch(&self, _request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError> {
        self.lock_responses()
            .pop_front()
            .ok_or_else(|| PortError::Transport("shape fixture exhausted".to_owned()))
    }
}

impl QueueFacade {
    fn lock_responses(&self) -> MutexGuard<'_, VecDeque<ShapeResponse>> {
        match self.responses.lock() {
            Ok(responses) => responses,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

fn streamed_response(body: ShapeBodyStream) -> ShapeResponse {
    ShapeResponse::streamed(
        200,
        vec![
            ShapeHeader::new("electric-handle", b"timing-handle"),
            ShapeHeader::new("electric-offset", b"20_0"),
        ],
        body,
    )
}

fn use_case(
    timeline: &Arc<Timeline>,
    post_open: PostOpenAuthority,
    responses: VecDeque<ShapeResponse>,
    active_lease: Duration,
) -> ShapeUseCase<ScriptedAuthority> {
    let authority = Arc::new(ScriptedAuthority {
        calls: AtomicUsize::new(0),
        post_open,
        timeline: Arc::clone(timeline),
    });
    let facade = Arc::new(QueueFacade {
        responses: Mutex::new(responses),
    });
    let base = ShapeUseCase::new(catalog(), authority, facade as Arc<dyn ShapeFacade>);
    match base.with_active_lease(active_lease) {
        Ok(use_case) => use_case,
        Err(error) => panic!("invalid timing-test lease: {error}"),
    }
}

fn catalog() -> ShapeCatalog {
    match ShapeCatalog::from_json(
        r#"{"cases":{"table":"aso.cases","columns":["id","practice_id"],"allowed_params":[],"relation":"view","object_namespace":"practice","scope_column":"practice_id"}}"#,
    ) {
        Ok(catalog) => catalog,
        Err(error) => panic!("invalid timing-test catalog: {error}"),
    }
}

fn claims() -> VerifiedClaims {
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
        expires_at: NOW_EPOCH_SECONDS + 300,
    }
}

fn request(cursor: Cursor) -> ShapeRequest {
    ShapeRequest {
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(3)),
        subject: "ignored-client-subject".to_owned(),
        shape: "cases".to_owned(),
        params: Vec::new(),
        cursor,
        protocol: ShapeProtocol::default(),
    }
}

fn initial_request() -> ShapeRequest {
    request(Cursor::Initial)
}

fn continuation_request() -> ShapeRequest {
    request(Cursor::Resume {
        handle: "timing-handle".to_owned(),
        offset: "20_0".to_owned(),
    })
}

async fn open(use_case: &ShapeUseCase<ScriptedAuthority>) -> ShapeResponse {
    match use_case
        .execute(
            &claims(),
            initial_request(),
            request_time(NOW_EPOCH_SECONDS),
        )
        .await
    {
        Ok(response) => response,
        Err(error) => panic!("authorized timing response failed: {error}"),
    }
}

async fn settle_background() {
    for _ in 0..3 {
        tokio::task::yield_now().await;
    }
}

async fn observe_cancellation(response: &mut ShapeResponse, timeline: &Timeline) {
    let next = response.body.next().await;
    timeline.record(EventKind::CancellationObserved);
    assert!(matches!(
        next,
        Some(Err(PortError::PermissionDenied(message)))
            if message == "shape response authority lease ended"
    ));
    assert!(response.body.next().await.is_none());
}

async fn assert_handle_released(use_case: &ShapeUseCase<ScriptedAuthority>) {
    let Err(error) = use_case
        .execute(
            &claims(),
            continuation_request(),
            request_time(NOW_EPOCH_SECONDS),
        )
        .await
    else {
        panic!("cancelled response retained its provisional handle");
    };
    assert!(matches!(error, ShapeUseCaseError::HandleMismatch));
}

#[tokio::test(start_paused = true)]
async fn upstream_stall_cancels_at_the_monotonic_deadline_without_client_polling() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[], true));
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Allow,
        VecDeque::from([response]),
        TEST_LEASE,
    );
    let mut response = open(&use_case).await;
    settle_background().await;

    tokio::time::advance(TEST_LEASE).await;
    settle_background().await;
    observe_cancellation(&mut response, &timeline).await;

    assert_eq!(
        timeline.at(EventKind::UpstreamPending),
        Some(Duration::ZERO)
    );
    assert_eq!(timeline.at(EventKind::UpstreamDrop), Some(TEST_LEASE));
    assert_eq!(
        timeline.at(EventKind::CancellationObserved),
        Some(TEST_LEASE)
    );
    assert_handle_released(&use_case).await;
    timeline.report("upstream-stall");
}

#[tokio::test(start_paused = true)]
async fn client_backpressure_cancels_at_the_deadline_and_discards_buffered_frames() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[b"one", b"two", b"three"], false));
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Allow,
        VecDeque::from([response]),
        TEST_LEASE,
    );
    let mut response = open(&use_case).await;
    settle_background().await;

    assert_eq!(
        timeline.at(EventKind::UpstreamFrame(1)),
        Some(Duration::ZERO)
    );
    assert_eq!(
        timeline.at(EventKind::UpstreamFrame(2)),
        Some(Duration::ZERO)
    );
    tokio::time::advance(TEST_LEASE).await;
    settle_background().await;
    observe_cancellation(&mut response, &timeline).await;

    assert_eq!(timeline.at(EventKind::UpstreamDrop), Some(TEST_LEASE));
    assert_eq!(timeline.at(EventKind::ConsumerFrame(1)), None);
    assert_handle_released(&use_case).await;
    timeline.report("client-backpressure");
}

#[tokio::test(start_paused = true)]
async fn authority_timeout_cancels_at_revalidation_plus_timeout() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[], true));
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Pending,
        VecDeque::from([response]),
        MAX_TEST_LEASE,
    );
    let mut response = open(&use_case).await;
    settle_background().await;

    tokio::time::advance(REVALIDATION_INTERVAL).await;
    settle_background().await;
    assert_eq!(
        timeline.at(EventKind::AuthorityCheck(2)),
        Some(REVALIDATION_INTERVAL)
    );
    tokio::time::advance(AUTHORITY_TIMEOUT).await;
    settle_background().await;
    observe_cancellation(&mut response, &timeline).await;

    let expected = REVALIDATION_INTERVAL + AUTHORITY_TIMEOUT;
    assert_eq!(timeline.at(EventKind::UpstreamDrop), Some(expected));
    assert_eq!(timeline.at(EventKind::CancellationObserved), Some(expected));
    assert_handle_released(&use_case).await;
    timeline.report("authority-timeout");
}

#[tokio::test(start_paused = true)]
async fn authority_unavailability_cancels_at_the_first_revalidation() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[], true));
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Unavailable,
        VecDeque::from([response]),
        TEST_LEASE,
    );
    let mut response = open(&use_case).await;
    settle_background().await;

    tokio::time::advance(REVALIDATION_INTERVAL).await;
    settle_background().await;
    observe_cancellation(&mut response, &timeline).await;

    assert_eq!(
        timeline.at(EventKind::UpstreamDrop),
        Some(REVALIDATION_INTERVAL)
    );
    assert_eq!(
        timeline.at(EventKind::CancellationObserved),
        Some(REVALIDATION_INTERVAL)
    );
    assert_handle_released(&use_case).await;
    timeline.report("authority-unavailable");
}

#[tokio::test(start_paused = true)]
async fn consumer_drop_cancels_upstream_at_the_observed_drop_time() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[], true));
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Allow,
        VecDeque::from([response]),
        TEST_LEASE,
    );
    let response = open(&use_case).await;
    settle_background().await;

    let drop_time = Duration::from_millis(125);
    tokio::time::advance(drop_time).await;
    timeline.record(EventKind::ConsumerDrop);
    drop(response);
    settle_background().await;

    assert_eq!(timeline.at(EventKind::ConsumerDrop), Some(drop_time));
    assert_eq!(timeline.at(EventKind::UpstreamDrop), Some(drop_time));
    assert_handle_released(&use_case).await;
    timeline.report("consumer-drop");
}

#[tokio::test(start_paused = true)]
async fn normal_completion_records_ordered_frame_and_final_frame_times() {
    let timeline = Arc::new(Timeline::new());
    let response = streamed_response(timed_body(&timeline, &[b"one", b"two", b"three"], false));
    let follow_up = ShapeResponse::new(304, Vec::new(), Vec::new());
    let use_case = use_case(
        &timeline,
        PostOpenAuthority::Allow,
        VecDeque::from([response, follow_up]),
        TEST_LEASE,
    );
    let mut response = open(&use_case).await;

    for (index, expected) in [b"one".as_slice(), b"two", b"three"]
        .into_iter()
        .enumerate()
    {
        if index > 0 {
            tokio::time::advance(Duration::from_millis(100)).await;
        }
        let frame = response.body.next().await;
        let frame_number = index + 1;
        timeline.record(EventKind::ConsumerFrame(frame_number));
        assert!(matches!(frame, Some(Ok(bytes)) if bytes == Bytes::from_static(expected)));
    }
    assert!(response.body.next().await.is_none());
    timeline.record(EventKind::NormalComplete);

    assert_eq!(
        timeline.at(EventKind::ConsumerFrame(1)),
        Some(Duration::ZERO)
    );
    assert_eq!(
        timeline.at(EventKind::ConsumerFrame(2)),
        Some(Duration::from_millis(100))
    );
    assert_eq!(
        timeline.at(EventKind::ConsumerFrame(3)),
        Some(Duration::from_millis(200))
    );
    assert_eq!(
        timeline.at(EventKind::NormalComplete),
        Some(Duration::from_millis(200))
    );
    assert_eq!(
        timeline.at(EventKind::UpstreamDrop),
        Some(Duration::from_millis(200))
    );

    let continuation = use_case
        .execute(
            &claims(),
            continuation_request(),
            request_time(NOW_EPOCH_SECONDS),
        )
        .await;
    let Ok(mut continuation) = continuation else {
        panic!("normal completion did not commit its continuation handle");
    };
    assert_eq!(continuation.status, 304);
    assert!(continuation.body.next().await.is_none());
    timeline.report("normal-completion");
}
