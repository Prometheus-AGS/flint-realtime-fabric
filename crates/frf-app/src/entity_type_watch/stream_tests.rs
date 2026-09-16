use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use chrono::Utc;
use frf_domain::{
    CanonicalValue, ChangeOp, CommittedEntityMutation, EntityField, EntityKey, EntityTypeDelivery,
    EntityTypeSelector, KeyPart, SourcePosition, TenantId,
};
use frf_ports::VerifiedClaims;
use uuid::Uuid;

use super::*;
use crate::entity_type_watch::{TypeWatchStart, WatchCheckpoint};

struct SequencedAuthz {
    checks: AtomicUsize,
    allowed_checks: usize,
    delay: Duration,
}

#[async_trait]
impl AuthzProvider for SequencedAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        tokio::time::sleep(self.delay).await;
        Ok(self.checks.fetch_add(1, Ordering::SeqCst) < self.allowed_checks)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct TestIdentity(TenantId);

#[async_trait]
impl IdentityVerifier for TestIdentity {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: frf_domain::SessionId::new(),
            originating_session_id: None,
            tenant_id: self.0,
            subject: "reader".to_owned(),
            email: None,
            role: None,
            principal_type: None,
            agent_id: None,
            workflow_id: None,
            scope: None,
            roles: Vec::new(),
            authorization_revision: None,
            projection_revision: None,
            projection_ids: Vec::new(),
            expires_at: 9_999_999_999,
        })
    }
}

fn producer(allowed_checks: usize) -> WatchProducer<SequencedAuthz, TestIdentity> {
    let tenant = TenantId::from_uuid(Uuid::from_u128(9));
    let selector = EntityTypeSelector {
        schema: "public".to_owned(),
        name: "orders".to_owned(),
        projection: "default".to_owned(),
    };
    WatchProducer {
        authz: Arc::new(SequencedAuthz {
            checks: AtomicUsize::new(0),
            allowed_checks,
            delay: Duration::ZERO,
        }),
        identity: Arc::new(TestIdentity(tenant)),
        checkpoints: CheckpointCodec::new(&[7; 32], 1).unwrap(),
        request: EntityTypeWatchRequest {
            entity_type: selector.clone(),
            tenant_id: tenant,
            start: TypeWatchStart::Live,
            bearer_token: "token".to_owned(),
        },
        subject: "reader".to_owned(),
        subscribe_tuple: RelationTuple {
            tenant_id: tenant,
            subject: "reader".to_owned(),
            relation: "subscribe".to_owned(),
            object: selector.canonical_name(),
        },
        source_epoch: "expected".to_owned(),
        deadline: Instant::now() + Duration::from_secs(60),
        recheck: Duration::from_millis(10),
    }
}

fn checkpoint_frame() -> EntityTypeWatchFrame {
    EntityTypeWatchFrame::CheckpointAdvanced(WatchCheckpoint {
        version: 1,
        token: vec![1],
        generation: 1,
    })
}

fn test_delivery(
    producer: &WatchProducer<SequencedAuthz, TestIdentity>,
    epoch: &str,
) -> EntityTypeDelivery {
    let selector = &producer.request.entity_type;
    EntityTypeDelivery {
        mutation: CommittedEntityMutation {
            event_id: "frfevent:v1:test".to_owned(),
            schema: selector.schema.clone(),
            table: selector.name.clone(),
            projection: selector.projection.clone(),
            entity_type: "public.orders@default".to_owned(),
            tenant_id: producer.request.tenant_id,
            key: EntityKey {
                parts: vec![KeyPart {
                    column: "id".to_owned(),
                    value: CanonicalValue::Text("1".to_owned()),
                }],
                canonical_id: "frfkey:v1:1".to_owned(),
            },
            op: ChangeOp::Insert,
            record: Some(vec![EntityField {
                column: "label".to_owned(),
                value: CanonicalValue::Text("row".to_owned()),
            }]),
            unchanged_toast: Vec::new(),
            source: SourcePosition {
                epoch: epoch.to_owned(),
                commit_lsn: 10,
                transaction_index: 0,
            },
            committed_at: Utc::now(),
        },
        broker_partition: 0,
        broker_offset: 1,
    }
}

#[tokio::test]
async fn source_terminal_follows_already_queued_frames() {
    let (tx, rx) = mpsc::channel(1);
    tx.send(checkpoint_frame()).await.unwrap();
    drop(tx);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    terminal_tx.send_replace(Some(Terminal::Source("closed".to_owned())));
    let state = OutputState {
        rx,
        terminal_rx,
        task: None,
        terminated: false,
    };

    let (first, state) = next_output(state).await.unwrap();
    assert!(matches!(
        first,
        Ok(EntityTypeWatchFrame::CheckpointAdvanced(_))
    ));
    let (terminal, _) = next_output(state).await.unwrap();
    assert!(matches!(terminal, Err(PortError::Transport(_))));
}

#[tokio::test]
async fn permission_terminal_discards_buffered_protected_frames() {
    let (tx, rx) = mpsc::channel(1);
    tx.send(checkpoint_frame()).await.unwrap();
    let (terminal_tx, terminal_rx) = watch::channel(None);
    terminal_tx.send_replace(Some(Terminal::Permission("revoked".to_owned())));
    let state = OutputState {
        rx,
        terminal_rx,
        task: None,
        terminated: false,
    };

    let (terminal, _) = next_output(state).await.unwrap();
    assert!(matches!(terminal, Err(PortError::PermissionDenied(_))));
}

#[tokio::test]
async fn stalled_initial_snapshot_becomes_terminal_lag() {
    let (tx, _rx) = mpsc::channel(1);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    let sent = enqueue_initial(
        &tx,
        &terminal_tx,
        vec![checkpoint_frame(), checkpoint_frame()],
        &producer(2),
    )
    .await;

    assert!(!sent);
    assert!(matches!(
        terminal_rx.borrow().as_ref(),
        Some(Terminal::Frame(frame))
            if matches!(frame.as_ref(), EntityTypeWatchFrame::Lagged { .. })
    ));
}

#[tokio::test]
async fn revocation_stops_initial_frames_before_enqueue() {
    let (tx, mut rx) = mpsc::channel(2);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    let sent = enqueue_initial(
        &tx,
        &terminal_tx,
        vec![checkpoint_frame(), checkpoint_frame()],
        &producer(1),
    )
    .await;

    assert!(!sent);
    assert!(matches!(
        rx.try_recv(),
        Ok(EntityTypeWatchFrame::CheckpointAdvanced(_))
    ));
    assert!(rx.try_recv().is_err());
    assert!(matches!(
        terminal_rx.borrow().as_ref(),
        Some(Terminal::Permission(_))
    ));
}

#[tokio::test]
async fn resnapshot_terminal_is_emitted_once() {
    let (_tx, rx) = mpsc::channel(1);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    let state = OutputState {
        rx,
        terminal_rx,
        task: None,
        terminated: false,
    };
    let waiter = tokio::spawn(next_output(state));
    tokio::task::yield_now().await;
    terminal_tx.send_replace(Some(Terminal::Frame(Box::new(
        EntityTypeWatchFrame::ResnapshotRequired {
            reason: ResnapshotReason::SourceEpochChanged,
            message: "changed".to_owned(),
        },
    ))));
    let (_, state) = waiter.await.unwrap().unwrap();
    assert!(next_output(state).await.is_none());
}

#[tokio::test]
async fn source_epoch_change_requires_resnapshot_before_delivery() {
    let producer = producer(0);
    let terminal = delivery_frame(&producer, test_delivery(&producer, "changed"))
        .await
        .unwrap_err();
    assert!(matches!(
        terminal,
        Terminal::Frame(frame)
            if matches!(frame.as_ref(), EntityTypeWatchFrame::ResnapshotRequired {
                reason: ResnapshotReason::SourceEpochChanged,
                ..
            })
    ));
}

#[tokio::test]
async fn revoked_subscription_cannot_advance_an_out_of_scope_checkpoint() {
    let producer = producer(0);
    let mut delivery = test_delivery(&producer, "expected");
    delivery.mutation.tenant_id = TenantId::from_uuid(Uuid::from_u128(10));

    let terminal = delivery_frame(&producer, delivery).await.unwrap_err();
    assert!(matches!(terminal, Terminal::Permission(_)));
}

#[tokio::test]
async fn revocation_during_object_authorization_preempts_live_delivery() {
    let producer = producer(2);
    let delivery = test_delivery(&producer, "expected");
    let raw = Box::pin(stream::iter(vec![Ok(delivery)]));
    let (tx, mut rx) = mpsc::channel(1);
    let (terminal_tx, terminal_rx) = watch::channel(None);

    run_producer(raw, tx, terminal_tx, Vec::new(), producer).await;

    assert!(rx.try_recv().is_err());
    assert!(matches!(
        terminal_rx.borrow().as_ref(),
        Some(Terminal::Permission(message)) if message.contains("revoked")
    ));
}

#[tokio::test]
async fn token_expiry_preempts_live_delivery_authorization() {
    let mut producer = producer(usize::MAX);
    producer.authz = Arc::new(SequencedAuthz {
        checks: AtomicUsize::new(0),
        allowed_checks: usize::MAX,
        delay: Duration::from_millis(50),
    });
    producer.deadline = Instant::now() + Duration::from_millis(10);
    let delivery = test_delivery(&producer, "expected");
    let raw = Box::pin(stream::iter(vec![Ok(delivery)]));
    let (tx, mut rx) = mpsc::channel(1);
    let (terminal_tx, terminal_rx) = watch::channel(None);

    run_producer(raw, tx, terminal_tx, Vec::new(), producer).await;

    assert!(rx.try_recv().is_err());
    assert!(matches!(
        terminal_rx.borrow().as_ref(),
        Some(Terminal::Permission(message)) if message.contains("expired")
    ));
}

#[tokio::test]
async fn token_expiry_preempts_initial_frame_authorization() {
    let mut producer = producer(usize::MAX);
    producer.authz = Arc::new(SequencedAuthz {
        checks: AtomicUsize::new(0),
        allowed_checks: usize::MAX,
        delay: Duration::from_millis(50),
    });
    producer.deadline = Instant::now() + Duration::from_millis(10);
    let (tx, mut rx) = mpsc::channel(1);
    let (terminal_tx, terminal_rx) = watch::channel(None);

    let sent = enqueue_initial(&tx, &terminal_tx, vec![checkpoint_frame()], &producer).await;

    assert!(!sent);
    assert!(rx.try_recv().is_err());
    assert!(matches!(
        terminal_rx.borrow().as_ref(),
        Some(Terminal::Permission(message)) if message.contains("expired")
    ));
}
