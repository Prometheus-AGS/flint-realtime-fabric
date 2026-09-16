#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "support/entity_projection/mod.rs"]
mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use frf_app::EntityUseCase;
use frf_broker_iggy::IggyBroker;
use frf_domain::{ChangeOp, Channel, ChannelId, EntityChange, EntityId, TenantId};
use frf_gateway::entity_grpc_service::EntityGrpcService;
use frf_ports::{
    AuthzProvider, EntityProjectionSnapshot, EntityStore, IdentityVerifier, LogBroker, PortError,
    ProjectionApply, ProjectionCursor, RelationTuple, VerifiedClaims,
};
use frf_postgres_cdc::model::CdcMutation;
use frf_projection_surreal::SurrealEntityProjection;
use frf_proto::fv1;
use frf_proto::fv1::entity_service_server::EntityService as _;
use futures_util::StreamExt as _;
use tokio::time::{sleep, timeout};
use tonic::Request;

use support::{TENANT, TIMEOUT};

#[derive(Default)]
struct TestAuthz {
    allow: AtomicBool,
    checks: AtomicUsize,
}

impl TestAuthz {
    fn allowed() -> Self {
        Self {
            allow: AtomicBool::new(true),
            checks: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl AuthzProvider for TestAuthz {
    async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
        self.checks.fetch_add(1, Ordering::SeqCst);
        Ok(self.allow.load(Ordering::SeqCst))
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct TestIdentity;

#[async_trait]
impl IdentityVerifier for TestIdentity {
    async fn verify(&self, _token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: frf_domain::SessionId::new(),
            originating_session_id: None,
            tenant_id: TenantId::from_uuid(TENANT),
            subject: "c009-user".to_owned(),
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

type Service = EntityGrpcService<SurrealEntityProjection, TestAuthz, TestIdentity>;

fn service(store: Arc<SurrealEntityProjection>, authz: Arc<TestAuthz>) -> Service {
    EntityGrpcService::new(Arc::new(EntityUseCase::new(
        store,
        authz,
        Arc::new(TestIdentity),
    )))
}

fn grpc_request<T>(message: T) -> Request<T> {
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("authorization", "Bearer c009-token".parse().unwrap());
    request
}

async fn get(service: &Service, entity_id: &str) -> Option<fv1::EntityChange> {
    service
        .get_entity(grpc_request(fv1::GetEntityRequest {
            entity_id: entity_id.to_owned(),
            tenant_id: TENANT.to_string(),
        }))
        .await
        .expect("GetEntity RPC")
        .into_inner()
        .entity
}

async fn watch(service: &Service, entity_id: &str) -> EntityWatch {
    service
        .watch_entity(grpc_request(fv1::WatchEntityRequest {
            entity_id: entity_id.to_owned(),
            tenant_id: TENANT.to_string(),
        }))
        .await
        .expect("WatchEntity RPC")
        .into_inner()
}

type EntityWatch = <Service as fv1::entity_service_server::EntityService>::WatchEntityStream;

async fn connect_projection(database: &str) -> Arc<SurrealEntityProjection> {
    let (username, password) = support::surreal_credentials();
    Arc::new(
        SurrealEntityProjection::connect(
            &support::surreal_url(),
            &username,
            &password,
            "c009",
            database,
        )
        .await
        .expect("connect SurrealDB projection"),
    )
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL, Iggy, and SurrealDB fixtures"]
#[allow(clippy::too_many_lines)]
async fn committed_database_changes_reach_durable_v1_entity_reads_and_watches() {
    let postgres = support::connect_postgres().await;
    support::install_source(&postgres).await;
    let cancellation_store = connect_projection("transaction-cancel").await;
    let cancellation_entity = EntityId::new();
    let cancellation_tenant = TenantId::from_uuid(TENANT);
    let mut cancellation_change = EntityChange {
        entity_id: cancellation_entity,
        tenant_id: cancellation_tenant,
        entity_type: "c009.widgets@default".to_owned(),
        op: ChangeOp::Update,
        data: serde_json::json!({"counter": 1}),
        previous: None,
        session_id: None,
        timestamp: Utc::now(),
        version: 0,
    };
    let cancellation_cursor = ProjectionCursor {
        source_epoch: "c009-cancel-v1".to_owned(),
        commit_lsn: 1,
        transaction_index: 0,
        broker_offset: 0,
    };
    cancellation_store
        .apply_projection(cancellation_change.clone(), cancellation_cursor.clone())
        .await
        .expect_err("missing update base must fail and cancel its transaction");
    cancellation_change.op = ChangeOp::Insert;
    assert!(matches!(
        cancellation_store
            .apply_projection(cancellation_change, cancellation_cursor)
            .await
            .expect("insert after failed update must use a clean transaction"),
        ProjectionApply::Applied(_)
    ));
    let broker = Arc::new(
        IggyBroker::new(&support::iggy_url())
            .await
            .expect("connect Iggy fixture"),
    );
    broker
        .ensure_channel(Channel {
            id: ChannelId::WELL_KNOWN_ENTITIES,
            tenant_id: TenantId::from_uuid(TENANT),
            path: "entity/changes".to_owned(),
        })
        .await
        .expect("ensure entity channel");
    let mut observer = support::subscribe(broker.as_ref()).await;
    let cdc = support::start_cdc(Arc::clone(&broker)).await;

    let initial_store = connect_projection("initial").await;
    let authz = Arc::new(TestAuthz::allowed());
    let initial_service = service(Arc::clone(&initial_store), Arc::clone(&authz));

    postgres
        .execute(
            "INSERT INTO c009.widgets(tenant_id, record_key, label, counter)
             VALUES ($1::text::uuid, 'widget-a', 'first', 1)",
            &[&TENANT.to_string()],
        )
        .await
        .expect("insert source row");
    let first_envelope = support::receive(&mut observer).await;
    let first_mutation: CdcMutation =
        serde_json::from_value(first_envelope.payload).expect("decode first CDC event");
    let entity_id = support::entity_id(&first_mutation.key.canonical_id);
    let entity_id_text = entity_id.to_string();
    let initial_projector =
        support::start_projector(Arc::clone(&broker), Arc::clone(&initial_store)).await;
    let inserted = get(&initial_service, &entity_id_text)
        .await
        .expect("projector readiness must include retained backlog catch-up");
    assert_eq!(inserted.entity_type, "c009.widgets@default");

    let initial_change = initial_store
        .get_entity(entity_id, TenantId::from_uuid(TENANT))
        .await
        .expect("read initial projection")
        .expect("initial entity");
    let initial_cursor = initial_store
        .projection_checkpoint()
        .await
        .expect("read initial cursor")
        .expect("initial cursor");
    initial_projector.stop().await;

    let rebuilt_store = connect_projection("rebuilt").await;
    assert!(
        rebuilt_store
            .install_projection_snapshot(EntityProjectionSnapshot {
                entities: vec![initial_change],
                cursor: initial_cursor,
            })
            .await
            .expect("install source snapshot")
    );
    let rebuilt_service = service(Arc::clone(&rebuilt_store), Arc::clone(&authz));
    assert!(get(&rebuilt_service, &entity_id_text).await.is_some());
    let mut overlap_watch = watch(&rebuilt_service, &entity_id_text).await;
    let rebuilt_projector =
        support::start_projector(Arc::clone(&broker), Arc::clone(&rebuilt_store)).await;
    assert!(
        timeout(Duration::from_secs(1), overlap_watch.next())
            .await
            .is_err(),
        "inclusive snapshot/WAL overlap must be deduplicated"
    );

    postgres
        .execute(
            "UPDATE c009.widgets SET counter = 2 WHERE record_key = 'widget-a'",
            &[],
        )
        .await
        .expect("update source row");
    let updated = timeout(TIMEOUT, overlap_watch.next())
        .await
        .expect("watch update timeout")
        .expect("watch stream ended")
        .expect("watch update failed");
    assert_eq!(updated.op, fv1::ChangeOp::Update as i32);
    assert!(updated.data.as_ref().unwrap().fields.contains_key("label"));
    rebuilt_projector.stop().await;

    let restarted_store = connect_projection("rebuilt").await;
    let restarted_service = service(Arc::clone(&restarted_store), Arc::clone(&authz));
    let restarted = get(&restarted_service, &entity_id_text)
        .await
        .expect("restart must preserve entity state");
    assert_eq!(restarted.op, fv1::ChangeOp::Update as i32);
    let mut revoked_watch = watch(&restarted_service, &entity_id_text).await;
    let restarted_projector =
        support::start_projector(Arc::clone(&broker), Arc::clone(&restarted_store)).await;
    assert!(
        timeout(Duration::from_secs(1), revoked_watch.next())
            .await
            .is_err(),
        "restart overlap must not duplicate the last event"
    );
    authz.allow.store(false, Ordering::SeqCst);
    postgres
        .execute(
            "UPDATE c009.widgets SET counter = 3 WHERE record_key = 'widget-a'",
            &[],
        )
        .await
        .expect("write event after revocation");
    let denied = timeout(TIMEOUT, revoked_watch.next())
        .await
        .expect("revocation result timeout")
        .expect("revoked stream ended without status")
        .expect_err("revoked stream leaked a protected event");
    assert_eq!(denied.code(), tonic::Code::PermissionDenied);

    authz.allow.store(true, Ordering::SeqCst);
    let mut delete_watch = watch(&restarted_service, &entity_id_text).await;
    postgres
        .execute(
            "DELETE FROM c009.widgets WHERE record_key = 'widget-a'",
            &[],
        )
        .await
        .expect("delete source row");
    let deleted = timeout(TIMEOUT, delete_watch.next())
        .await
        .expect("delete watch timeout")
        .expect("delete watch ended")
        .expect("delete watch failed");
    assert_eq!(deleted.op, fv1::ChangeOp::Delete as i32);
    let deadline = tokio::time::Instant::now() + TIMEOUT;
    while get(&restarted_service, &entity_id_text).await.is_some() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "delete did not clear state"
        );
        sleep(Duration::from_millis(100)).await;
    }
    assert!(authz.checks.load(Ordering::SeqCst) >= 6);

    restarted_projector.stop().await;
    cdc.stop().await;
    println!(
        "ENTITY_PROJECTION_PASS get=true watch=true backlog_ready=true overlap_dedup=true restart=true delete=true per_event_auth=true transaction_cancel=true"
    );
}
