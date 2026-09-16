#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

use frf_domain::{ChannelId, EntityId, Offset, TenantId};
use frf_gateway::entity_projector::{EntityProjector, EntityProjectorError};
use frf_ports::{EventStream, LogBroker};
use frf_postgres_cdc::{CdcConfig, TableEnrollment, TenantMode};
use futures_util::StreamExt as _;
use sha2::{Digest as _, Sha256};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub const TENANT: Uuid = Uuid::from_u128(9);
pub const TIMEOUT: Duration = Duration::from_secs(20);

pub fn postgres_url() -> String {
    std::env::var("FRF_PROJECTION_TEST_POSTGRES_URL").expect("PostgreSQL fixture URL")
}

pub fn iggy_url() -> String {
    std::env::var("FRF_PROJECTION_TEST_IGGY_URL").expect("Iggy fixture URL")
}

pub fn surreal_url() -> String {
    std::env::var("FRF_PROJECTION_TEST_SURREAL_URL").expect("SurrealDB fixture URL")
}

pub fn surreal_credentials() -> (String, String) {
    (
        std::env::var("FRF_PROJECTION_SURREAL_USER").expect("SurrealDB user"),
        std::env::var("FRF_PROJECTION_SURREAL_PASSWORD").expect("SurrealDB password"),
    )
}

pub async fn connect_postgres() -> Client {
    let (client, connection) = tokio_postgres::connect(&postgres_url(), NoTls)
        .await
        .expect("connect PostgreSQL fixture");
    tokio::spawn(async move { connection.await.expect("PostgreSQL fixture connection") });
    client
}

pub async fn install_source(client: &Client) {
    client
        .batch_execute(
            "CREATE SCHEMA c009;
             CREATE TABLE c009.widgets (
               tenant_id uuid NOT NULL,
               record_key text PRIMARY KEY,
               label text NOT NULL,
               counter bigint NOT NULL
             );
             ALTER TABLE c009.widgets REPLICA IDENTITY FULL;
             CREATE PUBLICATION c009_publication FOR TABLE c009.widgets;",
        )
        .await
        .expect("install c009 source schema");
}

pub fn cdc_config() -> CdcConfig {
    CdcConfig::new(
        postgres_url(),
        "c009_projection",
        "c009_publication",
        TenantId::from_uuid(TENANT),
        "entity/changes",
    )
    .with_source_epoch("c009-projection-v1")
    .with_enrollments(vec![TableEnrollment {
        schema: "c009".to_owned(),
        table: "widgets".to_owned(),
        projection: "default".to_owned(),
        columns: ["tenant_id", "record_key", "label", "counter"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        tenant: TenantMode::Column {
            column: "tenant_id".to_owned(),
        },
        unsigned_columns: Vec::new(),
    }])
}

pub async fn subscribe<L: LogBroker>(broker: &L) -> EventStream {
    let deadline = tokio::time::Instant::now() + TIMEOUT;
    loop {
        match broker
            .subscribe(
                ChannelId::WELL_KNOWN_ENTITIES,
                format!("c009-observer-{}", Uuid::new_v4()),
                Offset::BEGINNING,
            )
            .await
        {
            Ok(stream) => return stream,
            Err(_) if tokio::time::Instant::now() < deadline => {
                sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("could not subscribe to projection channel: {error}"),
        }
    }
}

pub async fn receive(stream: &mut EventStream) -> frf_domain::EventEnvelope {
    timeout(TIMEOUT, stream.next())
        .await
        .expect("timed out waiting for CDC event")
        .expect("CDC stream ended")
        .expect("CDC stream error")
}

pub fn entity_id(canonical_key: &str) -> EntityId {
    let digest = Sha256::digest(canonical_key.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    EntityId::from_uuid(Uuid::from_bytes(bytes))
}

pub struct RunningProjector {
    shutdown: watch::Sender<bool>,
    handle: JoinHandle<Result<(), EntityProjectorError>>,
}

pub struct RunningCdc {
    shutdown: watch::Sender<bool>,
    handle: JoinHandle<Result<(), frf_postgres_cdc::consumer::CdcError>>,
}

impl RunningCdc {
    pub async fn stop(self) {
        self.shutdown.send_replace(true);
        timeout(TIMEOUT, self.handle)
            .await
            .expect("CDC shutdown timed out")
            .expect("CDC task panicked")
            .expect("CDC shutdown failed");
    }
}

pub async fn start_cdc<L>(broker: Arc<L>) -> RunningCdc
where
    L: LogBroker,
{
    let consumer = frf_postgres_cdc::PostgresCdcConsumer::new(cdc_config(), broker);
    let (shutdown, shutdown_rx) = watch::channel(false);
    let (ready_tx, mut ready_rx) = watch::channel(false);
    let handle = tokio::spawn(async move {
        consumer
            .run_until_shutdown_with_readiness(shutdown_rx, ready_tx)
            .await
    });
    timeout(TIMEOUT, async {
        while !*ready_rx.borrow_and_update() {
            ready_rx.changed().await.expect("CDC exited before ready");
        }
    })
    .await
    .expect("CDC readiness timed out");
    RunningCdc { shutdown, handle }
}

impl RunningProjector {
    pub async fn stop(self) {
        self.shutdown.send_replace(true);
        timeout(TIMEOUT, self.handle)
            .await
            .expect("projector shutdown timed out")
            .expect("projector task panicked")
            .expect("projector shutdown failed");
    }
}

pub async fn start_projector<L, S>(broker: Arc<L>, store: Arc<S>) -> RunningProjector
where
    L: LogBroker,
    S: frf_ports::EntityStore,
{
    let projector = EntityProjector::new(broker, store);
    let (shutdown, shutdown_rx) = watch::channel(false);
    let (ready_tx, mut ready_rx) = watch::channel(false);
    let handle =
        tokio::spawn(async move { projector.run_until_shutdown(shutdown_rx, ready_tx).await });
    timeout(TIMEOUT, async {
        while !*ready_rx.borrow_and_update() {
            ready_rx
                .changed()
                .await
                .expect("projector exited before ready");
        }
    })
    .await
    .expect("projector readiness timed out");
    RunningProjector { shutdown, handle }
}
