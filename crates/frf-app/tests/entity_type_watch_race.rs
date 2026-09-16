#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use frf_app::{
    EntityTypeWatchConfig, EntityTypeWatchFrame, EntityTypeWatchRequest, EntityTypeWatchUseCase,
    TypeWatchStart,
};
use frf_domain::{
    CanonicalValue, ChangeOp, CommittedEntityMutation, EntityChange, EntityField, EntityId,
    EntityKey, EntityTypeDelivery, EntityTypeSelector, KeyPart, Offset, SourcePosition, TenantId,
};
use frf_ports::{
    AuthzProvider, EntityChangeStream, EntityProjectionSnapshot, EntityStore,
    EntityTypeDeliveryStream, EntityTypeProjectionSnapshot, EntityTypeWatchSource,
    IdentityVerifier, PortError, ProjectionApply, ProjectionCursor, RelationTuple, VerifiedClaims,
};
use futures_util::{StreamExt as _, stream};
use tokio::sync::{Mutex, Notify};
use tokio::time::timeout;
use uuid::Uuid;

const WAIT: Duration = Duration::from_secs(2);

struct RaceSource {
    history: Mutex<Vec<EntityTypeDelivery>>,
}

#[async_trait]
impl EntityTypeWatchSource for RaceSource {
    async fn subscribe(
        &self,
        _consumer_id: String,
        from: Offset,
    ) -> Result<EntityTypeDeliveryStream, PortError> {
        let deliveries = self
            .history
            .lock()
            .await
            .iter()
            .filter(|delivery| delivery.broker_offset >= from.0)
            .cloned()
            .map(Ok)
            .collect::<Vec<_>>();
        Ok(Box::pin(stream::iter(deliveries).chain(stream::pending())))
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

struct RaceProjection {
    captured: Arc<Notify>,
    release: Arc<Notify>,
    snapshot: EntityTypeProjectionSnapshot,
}

#[async_trait]
impl EntityStore for RaceProjection {
    async fn get_entity(
        &self,
        _entity_id: EntityId,
        _tenant_id: TenantId,
    ) -> Result<Option<EntityChange>, PortError> {
        Ok(None)
    }

    async fn watch_entity(
        &self,
        _entity_id: EntityId,
        _tenant_id: TenantId,
    ) -> Result<EntityChangeStream, PortError> {
        Ok(Box::pin(stream::empty()))
    }

    async fn apply_projection(
        &self,
        _change: EntityChange,
        _cursor: ProjectionCursor,
    ) -> Result<ProjectionApply, PortError> {
        Err(PortError::Transport(
            "race projection is read only".to_owned(),
        ))
    }

    async fn install_projection_snapshot(
        &self,
        _snapshot: EntityProjectionSnapshot,
    ) -> Result<bool, PortError> {
        Err(PortError::Transport(
            "race projection is read only".to_owned(),
        ))
    }

    async fn projection_checkpoint(&self) -> Result<Option<ProjectionCursor>, PortError> {
        Ok(self.snapshot.cursor.clone())
    }

    async fn snapshot_entity_type(
        &self,
        _entity_type: &EntityTypeSelector,
        _tenant_id: TenantId,
    ) -> Result<EntityTypeProjectionSnapshot, PortError> {
        let snapshot = self.snapshot.clone();
        self.captured.notify_one();
        self.release.notified().await;
        Ok(snapshot)
    }
}

struct AllowAuthz;

#[async_trait]
impl AuthzProvider for AllowAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(true)
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct Identity(TenantId);

#[async_trait]
impl IdentityVerifier for Identity {
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

fn selector() -> EntityTypeSelector {
    EntityTypeSelector {
        schema: "public".to_owned(),
        name: "orders".to_owned(),
        projection: "default".to_owned(),
    }
}

fn delivery(offset: u64, tenant: TenantId, key: &str) -> EntityTypeDelivery {
    EntityTypeDelivery {
        mutation: CommittedEntityMutation {
            event_id: format!("frfevent:v1:{offset}"),
            schema: "public".to_owned(),
            table: "orders".to_owned(),
            projection: "default".to_owned(),
            entity_type: "public.orders@default".to_owned(),
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
                value: CanonicalValue::Text(key.to_owned()),
            }]),
            unchanged_toast: Vec::new(),
            source: SourcePosition {
                epoch: "race-epoch".to_owned(),
                commit_lsn: offset + 100,
                transaction_index: 0,
            },
            committed_at: Utc::now(),
        },
        broker_partition: 0,
        broker_offset: offset,
    }
}

type RaceUseCase = EntityTypeWatchUseCase<RaceSource, RaceProjection, AllowAuthz, Identity>;

async fn assert_pre_snapshot_checkpoint_replays_seed(
    app: &RaceUseCase,
    tenant: TenantId,
    checkpoint: frf_app::WatchCheckpoint,
) {
    let mut resumed = app
        .watch(EntityTypeWatchRequest {
            entity_type: selector(),
            tenant_id: tenant,
            start: TypeWatchStart::Resume(checkpoint),
            bearer_token: "token".to_owned(),
        })
        .await
        .expect("resume admission");
    let _accepted = resumed
        .next()
        .await
        .expect("accepted frame")
        .expect("frame");
    let replay = resumed
        .next()
        .await
        .expect("replayed mutation")
        .expect("frame");
    assert!(matches!(
        replay,
        EntityTypeWatchFrame::Mutation { delivery, .. } if delivery.broker_offset == 0
    ));
}

#[tokio::test]
async fn mutation_committed_between_snapshot_capture_and_subscribe_is_replayed() {
    let tenant = TenantId::from_uuid(Uuid::from_u128(8));
    let seed = delivery(0, tenant, "seed");
    let source = Arc::new(RaceSource {
        history: Mutex::new(vec![seed.clone()]),
    });
    let captured = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let projection = Arc::new(RaceProjection {
        captured: Arc::clone(&captured),
        release: Arc::clone(&release),
        snapshot: EntityTypeProjectionSnapshot {
            entities: vec![seed],
            cursor: Some(ProjectionCursor {
                source_epoch: "race-epoch".to_owned(),
                commit_lsn: 100,
                transaction_index: 0,
                broker_offset: 0,
            }),
        },
    });
    let app = EntityTypeWatchUseCase::new(
        Arc::clone(&source),
        projection,
        Arc::new(AllowAuthz),
        Arc::new(Identity(tenant)),
        EntityTypeWatchConfig {
            enrolled_types: vec![selector()],
            source_epoch: "race-epoch".to_owned(),
            checkpoint_key: vec![9; 32],
            checkpoint_generation: 1,
            retention_seconds: 86_400,
            buffer_capacity: 8,
            authority_recheck: Duration::from_secs(1),
        },
    )
    .expect("valid watch config");
    let task = tokio::spawn(async move {
        let stream = app
            .watch(EntityTypeWatchRequest {
                entity_type: selector(),
                tenant_id: tenant,
                start: TypeWatchStart::Snapshot,
                bearer_token: "token".to_owned(),
            })
            .await;
        (app, stream)
    });
    timeout(WAIT, captured.notified())
        .await
        .expect("snapshot must capture its cursor");
    source
        .history
        .lock()
        .await
        .push(delivery(1, tenant, "raced"));
    release.notify_one();
    let (app, stream) = task.await.expect("watch task");
    let mut stream = stream.expect("watch admission");
    let mut saw_snapshot = false;
    let mut accepted_checkpoint = None;
    loop {
        match timeout(WAIT, stream.next())
            .await
            .expect("frame timeout")
            .expect("stream ended")
            .expect("watch error")
        {
            EntityTypeWatchFrame::Accepted(accepted) => {
                accepted_checkpoint = Some(accepted.start_checkpoint);
            }
            EntityTypeWatchFrame::SnapshotRow(_) => saw_snapshot = true,
            EntityTypeWatchFrame::Mutation { delivery, .. } => {
                assert!(saw_snapshot);
                assert_eq!(delivery.mutation.key.canonical_id, "frfkey:v1:raced");
                break;
            }
            _ => {}
        }
    }
    drop(stream);
    assert_pre_snapshot_checkpoint_replays_seed(
        &app,
        tenant,
        accepted_checkpoint.expect("accepted checkpoint"),
    )
    .await;
}
