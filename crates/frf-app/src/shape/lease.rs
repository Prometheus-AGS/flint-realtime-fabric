//! Poll-independent authority ownership for protected shape response streams.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use frf_ports::{AuthzProvider, PortError, RelationTuple, ShapeBodyStream, ShapeLease};
use futures_util::{Stream, StreamExt as _};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Instant, MissedTickBehavior};

pub(super) const REVALIDATION_INTERVAL: Duration = Duration::from_millis(750);
pub(super) const AUTHORITY_TIMEOUT: Duration = Duration::from_millis(750);

const SETTLEMENT_PENDING: u8 = 0;
const SETTLEMENT_COMPLETED: u8 = 1;
const SETTLEMENT_CANCELLED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StreamOutcome {
    Completed,
    Cancelled,
}

struct StreamSettlement {
    state: AtomicU8,
    on_settle: Box<dyn Fn(StreamOutcome) + Send + Sync>,
}

impl StreamSettlement {
    fn new(on_settle: impl Fn(StreamOutcome) + Send + Sync + 'static) -> Self {
        Self {
            state: AtomicU8::new(SETTLEMENT_PENDING),
            on_settle: Box::new(on_settle),
        }
    }

    fn complete(&self) {
        self.settle(SETTLEMENT_COMPLETED, StreamOutcome::Completed);
    }

    fn cancel(&self) {
        self.settle(SETTLEMENT_CANCELLED, StreamOutcome::Cancelled);
    }

    fn settle(&self, state: u8, outcome: StreamOutcome) {
        if self
            .state
            .compare_exchange(
                SETTLEMENT_PENDING,
                state,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            (self.on_settle)(outcome);
        }
    }
}

pub(super) struct GrantLease<A> {
    authz: Arc<A>,
    tuple: RelationTuple,
    grant_deadline: Instant,
}

impl<A> GrantLease<A> {
    pub(super) const fn new(authz: Arc<A>, tuple: RelationTuple, grant_deadline: Instant) -> Self {
        Self {
            authz,
            tuple,
            grant_deadline,
        }
    }
}

#[async_trait::async_trait]
impl<A> ShapeLease for GrantLease<A>
where
    A: AuthzProvider,
{
    async fn revalidate(&self) -> Result<(), PortError> {
        if Instant::now() >= self.grant_deadline {
            return Err(PortError::PermissionDenied(
                "shape response grant expired".to_owned(),
            ));
        }
        match self.authz.check(&self.tuple).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(PortError::PermissionDenied(
                "shape response authority ended".to_owned(),
            )),
            Err(error) => Err(error),
        }
    }
}

pub(super) async fn revalidate_once(lease: &Arc<dyn ShapeLease>) -> Result<(), PortError> {
    tokio::time::timeout(AUTHORITY_TIMEOUT, lease.revalidate())
        .await
        .map_err(|_| PortError::Timeout)?
}

pub(super) fn protect(
    body: ShapeBodyStream,
    lease: Arc<dyn ShapeLease>,
    deadline: Instant,
    on_settle: impl Fn(StreamOutcome) + Send + Sync + 'static,
) -> ShapeBodyStream {
    protect_with_timing(
        body,
        lease,
        deadline,
        REVALIDATION_INTERVAL,
        AUTHORITY_TIMEOUT,
        on_settle,
    )
}

fn protect_with_timing(
    body: ShapeBodyStream,
    lease: Arc<dyn ShapeLease>,
    deadline: Instant,
    revalidation_interval: Duration,
    authority_timeout: Duration,
    on_settle: impl Fn(StreamOutcome) + Send + Sync + 'static,
) -> ShapeBodyStream {
    let (sender, receiver) = mpsc::channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    let settlement = Arc::new(StreamSettlement::new(on_settle));
    let task = tokio::spawn(produce(
        body,
        sender,
        ProducerControl {
            lease,
            deadline,
            revalidation_interval,
            authority_timeout,
            cancelled: Arc::clone(&cancelled),
            settlement: Arc::clone(&settlement),
        },
    ));
    Box::pin(ProtectedShapeStream {
        receiver,
        cancelled,
        cancellation_reported: false,
        deadline,
        settlement,
        task,
    })
}

enum BodyMessage {
    Frame(Result<Bytes, PortError>),
    Complete,
}

struct ProducerControl {
    lease: Arc<dyn ShapeLease>,
    deadline: Instant,
    revalidation_interval: Duration,
    authority_timeout: Duration,
    cancelled: Arc<AtomicBool>,
    settlement: Arc<StreamSettlement>,
}

struct ProtectedShapeStream {
    receiver: mpsc::Receiver<BodyMessage>,
    cancelled: Arc<AtomicBool>,
    cancellation_reported: bool,
    deadline: Instant,
    settlement: Arc<StreamSettlement>,
    task: JoinHandle<()>,
}

impl Stream for ProtectedShapeStream {
    type Item = Result<Bytes, PortError>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.cancellation_reported {
            return Poll::Ready(None);
        }
        if self.lease_ended() {
            return Poll::Ready(Some(self.cancelled_frame()));
        }

        let polled = Pin::new(&mut self.receiver).poll_recv(context);
        if self.lease_ended() {
            return Poll::Ready(Some(self.cancelled_frame()));
        }
        match polled {
            Poll::Ready(Some(BodyMessage::Frame(frame))) => Poll::Ready(Some(frame)),
            Poll::Ready(Some(BodyMessage::Complete)) => {
                self.settlement.complete();
                Poll::Ready(None)
            }
            Poll::Ready(None) => {
                self.settlement.cancel();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl ProtectedShapeStream {
    fn lease_ended(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) || Instant::now() >= self.deadline
    }

    fn cancelled_frame(&mut self) -> Result<Bytes, PortError> {
        self.cancelled.store(true, Ordering::Release);
        self.cancellation_reported = true;
        self.receiver.close();
        self.settlement.cancel();
        Err(PortError::PermissionDenied(
            "shape response authority lease ended".to_owned(),
        ))
    }
}

impl Drop for ProtectedShapeStream {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.receiver.close();
        self.settlement.cancel();
        self.task.abort();
    }
}

async fn produce(
    mut body: ShapeBodyStream,
    sender: mpsc::Sender<BodyMessage>,
    control: ProducerControl,
) {
    let first_check = Instant::now() + control.revalidation_interval;
    let mut checks = tokio::time::interval_at(first_check, control.revalidation_interval);
    checks.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut pending: Option<BodyMessage> = None;

    loop {
        tokio::select! {
            biased;
            () = tokio::time::sleep_until(control.deadline) => {
                cancel(&control.cancelled);
                control.settlement.cancel();
                return;
            }
            _ = checks.tick() => {
                if !revalidate_during_production(
                    &control.lease,
                    control.deadline,
                    control.authority_timeout,
                    &sender,
                ).await {
                    cancel(&control.cancelled);
                    control.settlement.cancel();
                    return;
                }
            }
            () = sender.closed() => {
                control.settlement.cancel();
                return;
            },
            permit = sender.reserve(), if pending.is_some() => {
                let Ok(permit) = permit else {
                    control.settlement.cancel();
                    return;
                };
                if let Some(message) = pending.take() {
                    match message {
                        BodyMessage::Frame(frame) => {
                            let terminal = frame.is_err();
                            if terminal {
                                control.settlement.cancel();
                            }
                            permit.send(BodyMessage::Frame(frame));
                            if terminal {
                                return;
                            }
                        }
                        BodyMessage::Complete => {
                            permit.send(BodyMessage::Complete);
                            return;
                        }
                    }
                }
            }
            frame = body.next(), if pending.is_none() => {
                pending = Some(if let Some(frame) = frame {
                    BodyMessage::Frame(frame)
                } else {
                    BodyMessage::Complete
                });
            }
        }
    }
}

async fn revalidate_during_production(
    lease: &Arc<dyn ShapeLease>,
    deadline: Instant,
    authority_timeout: Duration,
    sender: &mpsc::Sender<BodyMessage>,
) -> bool {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return false;
    }
    let check_timeout = authority_timeout.min(remaining);
    tokio::select! {
        biased;
        () = sender.closed() => false,
        result = tokio::time::timeout(check_timeout, lease.revalidate()) => {
            matches!(result, Ok(Ok(())))
        }
    }
}

fn cancel(cancelled: &AtomicBool) {
    cancelled.store(true, Ordering::Release);
}

#[cfg(test)]
#[path = "lease_regression.rs"]
mod regression_tests;

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use futures_util::stream;

    use super::*;

    struct DeniedLease {
        calls: AtomicUsize,
    }

    struct AllowedLease;

    struct PendingUntilDropped {
        dropped: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl ShapeLease for DeniedLease {
        async fn revalidate(&self) -> Result<(), PortError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(PortError::PermissionDenied("revoked".to_owned()))
        }
    }

    #[async_trait::async_trait]
    impl ShapeLease for AllowedLease {
        async fn revalidate(&self) -> Result<(), PortError> {
            Ok(())
        }
    }

    impl Stream for PendingUntilDropped {
        type Item = Result<Bytes, PortError>;

        fn poll_next(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for PendingUntilDropped {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::Release);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn background_revalidation_discards_a_backpressured_frame() {
        let lease = Arc::new(DeniedLease {
            calls: AtomicUsize::new(0),
        });
        let outcome = Arc::new(AtomicUsize::new(0));
        let body: ShapeBodyStream = Box::pin(stream::iter([
            Ok(Bytes::from_static(b"first")),
            Ok(Bytes::from_static(b"second")),
        ]));
        let observed_outcome = Arc::clone(&outcome);
        let mut protected = protect_with_timing(
            body,
            Arc::clone(&lease) as Arc<dyn ShapeLease>,
            Instant::now() + Duration::from_millis(100),
            Duration::from_millis(10),
            Duration::from_millis(10),
            move |settled| {
                observed_outcome.store(
                    usize::from(settled == StreamOutcome::Cancelled) + 1,
                    Ordering::Release,
                );
            },
        );

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(10)).await;
        tokio::task::yield_now().await;

        let result = protected.next().await;
        assert!(matches!(
            result,
            Some(Err(PortError::PermissionDenied(message)))
                if message == "shape response authority lease ended"
        ));
        assert_eq!(lease.calls.load(Ordering::SeqCst), 1);
        assert_eq!(outcome.load(Ordering::Acquire), 2);
        assert!(protected.next().await.is_none());
    }

    #[tokio::test]
    async fn normal_completion_preserves_frame_order_and_settles_once() {
        let completions = Arc::new(AtomicUsize::new(0));
        let observed_completions = Arc::clone(&completions);
        let body: ShapeBodyStream = Box::pin(stream::iter([
            Ok(Bytes::from_static(b"first")),
            Ok(Bytes::from_static(b"-second")),
            Ok(Bytes::from_static(b"-third")),
        ]));
        let mut protected = protect_with_timing(
            body,
            Arc::new(AllowedLease),
            Instant::now() + Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_millis(10),
            move |outcome| {
                assert_eq!(outcome, StreamOutcome::Completed);
                observed_completions.fetch_add(1, Ordering::AcqRel);
            },
        );

        let mut received = Vec::new();
        while let Some(frame) = protected.next().await {
            received.extend_from_slice(&frame.expect("authorized body frame"));
        }
        drop(protected);

        assert_eq!(received, b"first-second-third");
        assert_eq!(completions.load(Ordering::Acquire), 1);
    }

    #[tokio::test]
    async fn dropping_the_consumer_drops_a_stalled_upstream_body() {
        let dropped = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        let body: ShapeBodyStream = Box::pin(PendingUntilDropped {
            dropped: Arc::clone(&dropped),
        });
        let observed_cancelled = Arc::clone(&cancelled);
        let protected = protect_with_timing(
            body,
            Arc::new(AllowedLease),
            Instant::now() + Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_millis(10),
            move |outcome| {
                observed_cancelled.store(outcome == StreamOutcome::Cancelled, Ordering::Release);
            },
        );
        tokio::task::yield_now().await;

        drop(protected);
        tokio::task::yield_now().await;

        assert!(dropped.load(Ordering::Acquire));
        assert!(cancelled.load(Ordering::Acquire));
    }
}
