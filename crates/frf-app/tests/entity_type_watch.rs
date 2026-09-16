#![allow(clippy::expect_used, clippy::unwrap_used)]
#[path = "support/entity_type_projection.rs"]
mod entity_type_projection;

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use frf_app::{
    AppError, EntityTypeWatchConfig, EntityTypeWatchFrame, EntityTypeWatchRequest,
    EntityTypeWatchUseCase, ResnapshotReason, TypeWatchStart,
};
use frf_domain::{
    CanonicalValue, ChangeOp, CommittedEntityMutation, EntityField, EntityKey, EntityTypeDelivery,
    EntityTypeSelector, KeyPart, Offset, SourcePosition, TenantId,
};
use frf_ports::{
    AuthzProvider, EntityTypeDeliveryStream, EntityTypeWatchSource, IdentityVerifier, PortError,
    RelationTuple, VerifiedClaims,
};
use futures_util::{StreamExt as _, stream};
use tokio::sync::{Mutex, Notify, broadcast};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use entity_type_projection::TestProjection;
const WAIT: Duration = Duration::from_secs(2);
struct TestSource {
    history: Arc<Mutex<Vec<EntityTypeDelivery>>>,
    sender: broadcast::Sender<EntityTypeDelivery>,
    earliest: AtomicU64,
    active: Arc<AtomicUsize>,
}

impl TestSource {
    fn new(history: Vec<EntityTypeDelivery>) -> Self {
        let (sender, _) = broadcast::channel(64);
        Self {
            history: Arc::new(Mutex::new(history)),
            sender,
            earliest: AtomicU64::new(0),
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    async fn publish(&self, delivery: EntityTypeDelivery) {
        self.history.lock().await.push(delivery.clone());
        let _ = self.sender.send(delivery);
    }
}

struct ActiveGuard(Arc<AtomicUsize>);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl EntityTypeWatchSource for TestSource {
    async fn subscribe(
        &self,
        _consumer_id: String,
        from: Offset,
    ) -> Result<EntityTypeDeliveryStream, PortError> {
        if from.0 < self.earliest.load(Ordering::SeqCst) {
            return Err(PortError::NotFound(
                "resnapshot_required: retained history expired".to_owned(),
            ));
        }
        let mut receiver = self.sender.subscribe();
        let history = self
            .history
            .lock()
            .await
            .iter()
            .filter(|delivery| delivery.broker_offset >= from.0)
            .cloned()
            .map(Ok)
            .collect::<Vec<_>>();
        self.active.fetch_add(1, Ordering::SeqCst);
        let guard = ActiveGuard(Arc::clone(&self.active));
        let live = async_stream::stream! {
            let _guard = guard;
            loop {
                match receiver.recv().await {
                    Ok(delivery) if delivery.broker_offset >= from.0 => yield Ok(delivery),
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        yield Err(PortError::Transport("test source lagged".to_owned()));
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Box::pin(stream::iter(history).chain(live)))
    }

    async fn head_offset(&self) -> Result<Option<Offset>, PortError> {
        Ok(self
            .history
            .lock()
            .await
            .last()
            .map(|delivery| Offset(delivery.broker_offset)))
    }
}

struct TestAuthz {
    subscribe: AtomicBool,
    denied_keys: HashSet<String>,
    view_block: Option<Arc<ViewBlock>>,
}

struct ViewBlock {
    entered: Notify,
    release: Notify,
}

#[async_trait]
impl AuthzProvider for TestAuthz {
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        if tuple.relation == "subscribe" {
            Ok(self.subscribe.load(Ordering::SeqCst) && tuple.subject != "blocked")
        } else {
            if let Some(block) = &self.view_block {
                block.entered.notify_one();
                block.release.notified().await;
            }
            Ok(!self.denied_keys.contains(&tuple.object))
        }
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

#[derive(Clone)]
struct TestIdentity {
    tenant: TenantId,
    subject: String,
}

#[async_trait]
impl IdentityVerifier for TestIdentity {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: frf_domain::SessionId::new(),
            originating_session_id: None,
            tenant_id: self.tenant,
            subject: self.subject.clone(),
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

fn selector(name: &str) -> EntityTypeSelector {
    EntityTypeSelector {
        schema: "c010".to_owned(),
        name: name.to_owned(),
        projection: "default".to_owned(),
    }
}

fn delivery(
    offset: u64,
    tenant: TenantId,
    kind: &EntityTypeSelector,
    key: &str,
) -> EntityTypeDelivery {
    EntityTypeDelivery {
        mutation: CommittedEntityMutation {
            event_id: format!("frfevent:v1:{offset}"),
            schema: kind.schema.clone(),
            table: kind.name.clone(),
            projection: kind.projection.clone(),
            entity_type: kind.canonical_name(),
            tenant_id: tenant,
            key: EntityKey {
                parts: vec![KeyPart {
                    column: "id".to_owned(),
                    value: CanonicalValue::Text(key.to_owned()),
                }],
                canonical_id: format!("frfkey:v1:{key}"),
            },
            op: ChangeOp::Insert,
            record: Some(vec![EntityField {
                column: "label".to_owned(),
                value: CanonicalValue::Text(format!("row-{key}")),
            }]),
            unchanged_toast: Vec::new(),
            source: SourcePosition {
                epoch: "c010-epoch".to_owned(),
                commit_lsn: offset + 100,
                transaction_index: 0,
            },
            committed_at: Utc::now(),
        },
        broker_partition: 0,
        broker_offset: offset,
    }
}

fn usecase(
    source: Arc<TestSource>,
    authz: Arc<TestAuthz>,
    identity: TestIdentity,
    buffer_capacity: usize,
) -> EntityTypeWatchUseCase<TestSource, TestProjection, TestAuthz, TestIdentity> {
    let projection = Arc::new(TestProjection::new(Arc::clone(&source.history)));
    EntityTypeWatchUseCase::new(
        source,
        projection,
        authz,
        Arc::new(identity),
        EntityTypeWatchConfig {
            enrolled_types: vec![selector("orders")],
            source_epoch: "c010-epoch".to_owned(),
            checkpoint_key: b"c010-test-checkpoint-key-at-least-32-bytes".to_vec(),
            checkpoint_generation: 7,
            retention_seconds: 86_400,
            buffer_capacity,
            authority_recheck: Duration::from_millis(25),
        },
    )
    .expect("valid watch config")
}

fn request(tenant: TenantId, start: TypeWatchStart) -> EntityTypeWatchRequest {
    EntityTypeWatchRequest {
        entity_type: selector("orders"),
        tenant_id: tenant,
        start,
        bearer_token: "token".to_owned(),
    }
}

async fn next_mutation(
    stream: &mut frf_app::EntityTypeWatchStream,
) -> (EntityTypeDelivery, frf_app::WatchCheckpoint) {
    loop {
        let frame = timeout(WAIT, stream.next())
            .await
            .expect("watch timeout")
            .expect("watch ended")
            .expect("watch error");
        if let EntityTypeWatchFrame::Mutation {
            delivery,
            checkpoint,
        } = frame
        {
            return (delivery, checkpoint);
        }
    }
}

#[tokio::test]
async fn authorized_subscribers_share_type_history_without_cross_scope_payloads() {
    let tenant = TenantId::from_uuid(Uuid::from_u128(1));
    let other_tenant = TenantId::from_uuid(Uuid::from_u128(2));
    let orders = selector("orders");
    let source = Arc::new(TestSource::new(vec![
        delivery(0, tenant, &orders, "allowed"),
        delivery(1, tenant, &selector("notes"), "other-type"),
        delivery(2, other_tenant, &orders, "other-tenant"),
        delivery(3, tenant, &orders, "denied"),
    ]));
    let authz = Arc::new(TestAuthz {
        subscribe: AtomicBool::new(true),
        denied_keys: ["frfkey:v1:denied".to_owned()].into_iter().collect(),
        view_block: None,
    });
    let app = usecase(
        Arc::clone(&source),
        Arc::clone(&authz),
        TestIdentity {
            tenant,
            subject: "reader".to_owned(),
        },
        8,
    );
    let mut first = app
        .watch(request(tenant, TypeWatchStart::Snapshot))
        .await
        .unwrap();
    let mut second = app
        .watch(request(tenant, TypeWatchStart::Snapshot))
        .await
        .unwrap();
    for stream in [&mut first, &mut second] {
        let mut snapshot_keys = Vec::new();
        loop {
            match timeout(WAIT, stream.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap()
            {
                EntityTypeWatchFrame::SnapshotRow(row) => snapshot_keys.push(row.key.canonical_id),
                EntityTypeWatchFrame::SnapshotComplete {
                    barrier, row_count, ..
                } => {
                    assert_eq!(row_count, 1);
                    assert_eq!(barrier.commit_lsn, 100);
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(snapshot_keys, ["frfkey:v1:allowed"]);
    }

    source.publish(delivery(4, tenant, &orders, "live")).await;
    assert_eq!(
        next_mutation(&mut first).await.0.mutation.key.canonical_id,
        "frfkey:v1:live"
    );
    assert_eq!(
        next_mutation(&mut second).await.0.mutation.key.canonical_id,
        "frfkey:v1:live"
    );

    let blocked = usecase(
        Arc::clone(&source),
        authz,
        TestIdentity {
            tenant,
            subject: "blocked".to_owned(),
        },
        8,
    );
    assert!(matches!(
        blocked.watch(request(tenant, TypeWatchStart::Live)).await,
        Err(AppError::Forbidden(_))
    ));

    drop(first);
    drop(second);
    timeout(WAIT, async {
        while source.active.load(Ordering::SeqCst) != 0 {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("subscriptions must tear down");
}

#[tokio::test]
async fn resume_lag_and_idle_revocation_are_explicit() {
    let tenant = TenantId::from_uuid(Uuid::from_u128(3));
    let orders = selector("orders");
    let source = Arc::new(TestSource::new(vec![delivery(0, tenant, &orders, "seed")]));
    let authz = Arc::new(TestAuthz {
        subscribe: AtomicBool::new(true),
        denied_keys: HashSet::new(),
        view_block: None,
    });
    let app = usecase(
        Arc::clone(&source),
        Arc::clone(&authz),
        TestIdentity {
            tenant,
            subject: "reader".to_owned(),
        },
        1,
    );
    let mut original = app
        .watch(request(tenant, TypeWatchStart::Live))
        .await
        .unwrap();
    let accepted = timeout(WAIT, original.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let EntityTypeWatchFrame::Accepted(accepted) = accepted else {
        panic!("accepted frame must be first");
    };
    source.publish(delivery(1, tenant, &orders, "one")).await;
    let (_, checkpoint) = next_mutation(&mut original).await;
    drop(original);

    let mut resumed = app
        .watch(request(tenant, TypeWatchStart::Resume(checkpoint.clone())))
        .await
        .unwrap();
    let _accepted = timeout(WAIT, resumed.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    source.publish(delivery(2, tenant, &orders, "two")).await;
    assert_eq!(next_mutation(&mut resumed).await.0.broker_offset, 2);
    drop(resumed);

    source.earliest.store(3, Ordering::SeqCst);
    let mut expired = app
        .watch(request(tenant, TypeWatchStart::Resume(checkpoint)))
        .await
        .unwrap();
    assert!(matches!(
        expired.next().await.unwrap().unwrap(),
        EntityTypeWatchFrame::ResnapshotRequired {
            reason: ResnapshotReason::HistoryExpired,
            ..
        }
    ));

    source.earliest.store(0, Ordering::SeqCst);
    let mut lagged = app
        .watch(request(tenant, TypeWatchStart::Live))
        .await
        .unwrap();
    let _accepted = timeout(WAIT, lagged.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    source.publish(delivery(3, tenant, &orders, "three")).await;
    source.publish(delivery(4, tenant, &orders, "four")).await;
    sleep(Duration::from_millis(25)).await;
    assert!(matches!(
        lagged.next().await.unwrap().unwrap(),
        EntityTypeWatchFrame::Lagged {
            checkpoint_resumable: false,
            ..
        }
    ));

    let mut revoked = app
        .watch(request(tenant, TypeWatchStart::Live))
        .await
        .unwrap();
    let _accepted = timeout(WAIT, revoked.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    authz.subscribe.store(false, Ordering::SeqCst);
    let denied = timeout(WAIT, revoked.next()).await.unwrap().unwrap();
    assert!(matches!(denied, Err(PortError::PermissionDenied(_))));
    assert!(accepted.start_checkpoint.token.len() > 32);
}

#[tokio::test]
async fn cancellation_during_awaited_authorization_releases_subscription() {
    let tenant = TenantId::from_uuid(Uuid::from_u128(4));
    let orders = selector("orders");
    let source = Arc::new(TestSource::new(vec![delivery(0, tenant, &orders, "seed")]));
    let block = Arc::new(ViewBlock {
        entered: Notify::new(),
        release: Notify::new(),
    });
    let app = usecase(
        Arc::clone(&source),
        Arc::new(TestAuthz {
            subscribe: AtomicBool::new(true),
            denied_keys: HashSet::new(),
            view_block: Some(Arc::clone(&block)),
        }),
        TestIdentity {
            tenant,
            subject: "reader".to_owned(),
        },
        8,
    );
    let task =
        tokio::spawn(async move { app.watch(request(tenant, TypeWatchStart::Snapshot)).await });
    timeout(WAIT, block.entered.notified())
        .await
        .expect("object authorization must start");
    task.abort();
    timeout(WAIT, async {
        while source.active.load(Ordering::SeqCst) != 0 {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelled authorization must release its source subscription");
}
