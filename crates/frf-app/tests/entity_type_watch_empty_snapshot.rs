#![allow(clippy::expect_used, clippy::unwrap_used)]
#[path = "support/entity_type_projection.rs"]
mod entity_type_projection;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use entity_type_projection::TestProjection;
use frf_app::{
    EntityTypeWatchConfig, EntityTypeWatchFrame, EntityTypeWatchRequest, EntityTypeWatchUseCase,
    TypeWatchStart,
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
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

const WAIT: Duration = Duration::from_secs(2);

struct HistoricalSource {
    history: Arc<Mutex<Vec<EntityTypeDelivery>>>,
    earliest: u64,
}

#[async_trait]
impl EntityTypeWatchSource for HistoricalSource {
    async fn subscribe(
        &self,
        _consumer_id: String,
        from: Offset,
    ) -> Result<EntityTypeDeliveryStream, PortError> {
        if from.0 < self.earliest {
            return Err(PortError::NotFound(
                "resnapshot_required: retained history expired".to_owned(),
            ));
        }
        let deliveries = self
            .history
            .lock()
            .await
            .iter()
            .filter(|delivery| delivery.broker_offset >= from.0)
            .cloned()
            .map(Ok)
            .collect::<Vec<_>>();
        Ok(Box::pin(stream::iter(deliveries)))
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

struct GrantableAuthz {
    view_granted: AtomicBool,
}

#[async_trait]
impl AuthzProvider for GrantableAuthz {
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(tuple.relation == "subscribe" || self.view_granted.load(Ordering::SeqCst))
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct Identity {
    tenant: TenantId,
}

#[async_trait]
impl IdentityVerifier for Identity {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: frf_domain::SessionId::new(),
            originating_session_id: None,
            tenant_id: self.tenant,
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
        schema: "c010".to_owned(),
        name: "orders".to_owned(),
        projection: "default".to_owned(),
    }
}

fn delivery(tenant: TenantId, offset: u64) -> EntityTypeDelivery {
    let entity_type = selector();
    EntityTypeDelivery {
        mutation: CommittedEntityMutation {
            event_id: "frfevent:v1:empty-snapshot-grant".to_owned(),
            schema: entity_type.schema.clone(),
            table: entity_type.name.clone(),
            projection: entity_type.projection.clone(),
            entity_type: entity_type.canonical_name(),
            tenant_id: tenant,
            key: EntityKey {
                parts: vec![KeyPart {
                    column: "id".to_owned(),
                    value: CanonicalValue::Text("previously-denied".to_owned()),
                }],
                canonical_id: "frfkey:v1:previously-denied".to_owned(),
            },
            op: ChangeOp::Insert,
            record: Some(vec![EntityField {
                column: "label".to_owned(),
                value: CanonicalValue::Text("now-visible".to_owned()),
            }]),
            unchanged_toast: Vec::new(),
            source: SourcePosition {
                epoch: "c010-epoch".to_owned(),
                commit_lsn: 100,
                transaction_index: 0,
            },
            committed_at: Utc::now(),
        },
        broker_partition: 0,
        broker_offset: offset,
    }
}

fn request(tenant: TenantId, start: TypeWatchStart) -> EntityTypeWatchRequest {
    EntityTypeWatchRequest {
        entity_type: selector(),
        tenant_id: tenant,
        start,
        bearer_token: "token".to_owned(),
    }
}

#[tokio::test]
async fn fully_filtered_snapshot_resume_replays_rows_after_a_grant() {
    let tenant = TenantId::from_uuid(Uuid::from_u128(17));
    let history = Arc::new(Mutex::new(vec![delivery(tenant, 5)]));
    let source = Arc::new(HistoricalSource {
        history: Arc::clone(&history),
        earliest: 5,
    });
    let authz = Arc::new(GrantableAuthz {
        view_granted: AtomicBool::new(false),
    });
    let app = EntityTypeWatchUseCase::new(
        Arc::clone(&source),
        Arc::new(TestProjection::new(history)),
        Arc::clone(&authz),
        Arc::new(Identity { tenant }),
        EntityTypeWatchConfig {
            enrolled_types: vec![selector()],
            source_epoch: "c010-epoch".to_owned(),
            checkpoint_key: vec![17; 32],
            checkpoint_generation: 1,
            retention_seconds: 86_400,
            buffer_capacity: 8,
            authority_recheck: Duration::from_secs(1),
        },
    )
    .expect("valid watch config");

    let mut snapshot = app
        .watch(request(tenant, TypeWatchStart::Snapshot))
        .await
        .expect("snapshot admission");
    let completion = loop {
        let frame = timeout(WAIT, snapshot.next())
            .await
            .expect("snapshot timeout")
            .expect("snapshot ended")
            .expect("snapshot frame");
        if let EntityTypeWatchFrame::SnapshotComplete {
            checkpoint,
            row_count,
            ..
        } = frame
        {
            assert_eq!(row_count, 0);
            break checkpoint;
        }
    };
    drop(snapshot);

    authz.view_granted.store(true, Ordering::SeqCst);
    let mut resumed = app
        .watch(request(tenant, TypeWatchStart::Resume(completion)))
        .await
        .expect("resume admission");
    loop {
        let frame = timeout(WAIT, resumed.next())
            .await
            .expect("resume timeout")
            .expect("resume ended")
            .expect("resume frame");
        if let EntityTypeWatchFrame::Mutation { delivery, .. } = frame {
            assert_eq!(delivery.broker_offset, 5);
            assert_eq!(
                delivery.mutation.key.canonical_id,
                "frfkey:v1:previously-denied"
            );
            break;
        }
    }
}
