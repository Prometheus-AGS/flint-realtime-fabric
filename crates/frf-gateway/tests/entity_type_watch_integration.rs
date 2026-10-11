#![allow(clippy::expect_used, clippy::unwrap_used)]

#[allow(dead_code)]
#[path = "support/entity_projection/mod.rs"]
mod support;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use frf_app::{EntityTypeWatchConfig, EntityTypeWatchUseCase};
use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, ChannelId, EntityTypeSelector, TenantId};
use frf_gateway::entity_type_grpc_service::EntityTypeGrpcService;
use frf_ports::{
    AuthzProvider, EntityStore, IdentityVerifier, LogBroker, PortError, RelationTuple,
    VerifiedClaims,
};
use frf_projection_surreal::SurrealEntityProjection;
use frf_proto::fv2::{
    self, SnapshotStart, WatchEntityTypeRequest, entity_service_client::EntityServiceClient,
    watch_entity_type_request, watch_entity_type_response,
};
use frf_watch_broker::BrokerEntityTypeWatchSource;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_postgres::Client;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, transport::Server};

use support::{TENANT, TIMEOUT};

struct WatchAuthz;

#[async_trait]
impl AuthzProvider for WatchAuthz {
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        Ok(tuple.subject != "blocked")
    }

    async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }

    async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
        Ok(())
    }
}

struct WatchIdentity;

#[async_trait]
impl IdentityVerifier for WatchIdentity {
    async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError> {
        Ok(VerifiedClaims {
            session_id: frf_domain::SessionId::new(),
            originating_session_id: None,
            tenant_id: TenantId::from_uuid(TENANT),
            subject: token.to_owned(),
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
        schema: "c009".to_owned(),
        name: "widgets".to_owned(),
        projection: "default".to_owned(),
    }
}

fn watch_request(
    token: &str,
    start: watch_entity_type_request::Start,
) -> Request<WatchEntityTypeRequest> {
    let mut request = Request::new(WatchEntityTypeRequest {
        entity_type: Some(fv2::EntityType {
            schema: "c009".to_owned(),
            name: "widgets".to_owned(),
            projection: "default".to_owned(),
        }),
        tenant_id: TENANT.to_string(),
        start: Some(start),
    });
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {token}")
            .parse()
            .expect("authorization metadata"),
    );
    request
}

async fn next_frame(
    stream: &mut tonic::Streaming<fv2::WatchEntityTypeResponse>,
) -> watch_entity_type_response::Frame {
    timeout(TIMEOUT, stream.message())
        .await
        .expect("watch response timeout")
        .expect("watch transport status")
        .expect("watch ended")
        .frame
        .expect("watch frame")
}

async fn next_mutation(
    stream: &mut tonic::Streaming<fv2::WatchEntityTypeResponse>,
) -> fv2::EntityMutation {
    loop {
        if let watch_entity_type_response::Frame::Mutation(mutation) = next_frame(stream).await {
            return mutation;
        }
    }
}

async fn connect_projection() -> Arc<SurrealEntityProjection> {
    let (username, password) = support::surreal_credentials();
    Arc::new(
        SurrealEntityProjection::connect(
            &support::surreal_url(),
            &username,
            &password,
            "c010",
            "type-watch",
        )
        .await
        .expect("connect type-watch projection"),
    )
}

async fn client(address: std::net::SocketAddr) -> EntityServiceClient<tonic::transport::Channel> {
    EntityServiceClient::connect(format!("http://{address}"))
        .await
        .expect("connect generated v2 client")
}

struct RunningFixture {
    postgres: Client,
    store: Arc<SurrealEntityProjection>,
    address: std::net::SocketAddr,
    shutdown: oneshot::Sender<()>,
    server: JoinHandle<()>,
    projector: support::RunningProjector,
    cdc: support::RunningCdc,
}

async fn start_fixture() -> RunningFixture {
    let postgres = support::connect_postgres().await;
    support::install_source(&postgres).await;
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
    let cdc = support::start_cdc(Arc::clone(&broker)).await;
    let store = connect_projection().await;
    let projector = support::start_projector(Arc::clone(&broker), Arc::clone(&store)).await;
    let app = EntityTypeWatchUseCase::new(
        Arc::new(BrokerEntityTypeWatchSource::new(broker)),
        Arc::clone(&store),
        Arc::new(WatchAuthz),
        Arc::new(WatchIdentity),
        EntityTypeWatchConfig {
            enrolled_types: vec![selector()],
            source_epoch: "c009-projection-v1".to_owned(),
            checkpoint_key: b"c010-integration-checkpoint-key-32-bytes-minimum".to_vec(),
            checkpoint_generation: 1,
            retention_seconds: 86_400,
            buffer_capacity: 16,
            authority_recheck: Duration::from_secs(1),
        },
    )
    .expect("valid watch configuration");
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind tonic listener");
    let address = listener.local_addr().expect("tonic listener address");
    let (shutdown, shutdown_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(EntityTypeGrpcService::new(Arc::new(app)).into_server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("serve v2 entity watch");
    });
    RunningFixture {
        postgres,
        store,
        address,
        shutdown,
        server,
        projector,
        cdc,
    }
}

async fn verify_durable_snapshot_and_denial(
    store: &SurrealEntityProjection,
    address: std::net::SocketAddr,
) {
    timeout(TIMEOUT, async {
        loop {
            let snapshot = store
                .snapshot_entity_type(&selector(), TenantId::from_uuid(TENANT))
                .await
                .expect("read typed projection");
            if snapshot.entities.len() == 1 {
                break;
            }
            sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("typed projection catch-up");
    let mut snapshot_client = client(address).await;
    let mut rebuilt = snapshot_client
        .watch_entity_type(watch_request(
            "reader-c",
            watch_entity_type_request::Start::Snapshot(SnapshotStart {}),
        ))
        .await
        .expect("rebuilt snapshot admission")
        .into_inner();
    assert!(matches!(
        next_frame(&mut rebuilt).await,
        watch_entity_type_response::Frame::Accepted(_)
    ));
    assert!(matches!(
        next_frame(&mut rebuilt).await,
        watch_entity_type_response::Frame::SnapshotRow(_)
    ));
    assert!(matches!(
        next_frame(&mut rebuilt).await,
        watch_entity_type_response::Frame::SnapshotComplete(_)
    ));
    let denied = snapshot_client
        .watch_entity_type(watch_request(
            "blocked",
            watch_entity_type_request::Start::Snapshot(SnapshotStart {}),
        ))
        .await
        .expect_err("unauthorized same-tenant subject must fail admission");
    assert_eq!(denied.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL, Iggy, and SurrealDB fixtures"]
async fn committed_database_changes_cross_the_v2_tonic_watch_boundary() {
    let fixture = start_fixture().await;
    let RunningFixture {
        postgres,
        store,
        address,
        shutdown,
        server,
        projector,
        cdc,
    } = fixture;

    let mut first_client = client(address).await;
    let mut second_client = client(address).await;
    let snapshot = watch_entity_type_request::Start::Snapshot(SnapshotStart {});
    let mut first = first_client
        .watch_entity_type(watch_request("reader-a", snapshot.clone()))
        .await
        .expect("first watch admission")
        .into_inner();
    let mut second = second_client
        .watch_entity_type(watch_request("reader-b", snapshot))
        .await
        .expect("second watch admission")
        .into_inner();
    for stream in [&mut first, &mut second] {
        assert!(matches!(
            next_frame(stream).await,
            watch_entity_type_response::Frame::Accepted(_)
        ));
        assert!(matches!(
            next_frame(stream).await,
            watch_entity_type_response::Frame::SnapshotComplete(_)
        ));
    }

    postgres
        .execute(
            "INSERT INTO c009.widgets(tenant_id, record_key, label, counter)
             VALUES ($1::text::uuid, 'watch-a', 'first', 1)",
            &[&TENANT.to_string()],
        )
        .await
        .expect("insert source row");
    let first_insert = next_mutation(&mut first).await;
    let second_insert = next_mutation(&mut second).await;
    assert_eq!(first_insert.event_id, second_insert.event_id);
    assert_eq!(first_insert.tenant_id, TENANT.to_string());
    assert_eq!(first_insert.entity_type.as_ref().unwrap().name, "widgets");
    let checkpoint = first_insert.checkpoint.clone().expect("insert checkpoint");

    verify_durable_snapshot_and_denial(&store, address).await;

    let mut resume_client = client(address).await;
    let mut resumed = resume_client
        .watch_entity_type(watch_request(
            "reader-a",
            watch_entity_type_request::Start::Resume(checkpoint),
        ))
        .await
        .expect("resume admission")
        .into_inner();
    assert!(matches!(
        next_frame(&mut resumed).await,
        watch_entity_type_response::Frame::Accepted(_)
    ));
    postgres
        .execute("DELETE FROM c009.widgets WHERE record_key = 'watch-a'", &[])
        .await
        .expect("delete source row");
    let deleted = next_mutation(&mut resumed).await;
    assert_eq!(deleted.op, fv2::ChangeOp::Delete as i32);
    assert!(deleted.record.is_none());

    drop(first);
    drop(second);
    drop(resumed);
    let _ = shutdown.send(());
    timeout(TIMEOUT, server)
        .await
        .expect("tonic shutdown timeout")
        .expect("tonic task panicked");
    projector.stop().await;
    cdc.stop().await;
    println!(
        "ENTITY_TYPE_WATCH_PASS tonic=true subscribers=2 snapshot=true resume=true delete_key_only=true unauthorized_denied=true"
    );
}
