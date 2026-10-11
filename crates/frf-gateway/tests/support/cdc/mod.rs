#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

use frf_domain::{ChannelId, EventEnvelope, Offset, TenantId};
use frf_ports::{EventStream, LogBroker};
use frf_postgres_cdc::{CdcConfig, TableEnrollment, TenantMode};
use futures_util::StreamExt;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub const TENANT: Uuid = Uuid::from_u128(7);
pub const COLUMN_TENANT: Uuid = Uuid::from_u128(8);
pub const RECEIVE_TIMEOUT: Duration = Duration::from_secs(20);

pub fn database_url() -> String {
    std::env::var("FRF_CDC_TEST_POSTGRES_URL")
        .expect("FRF_CDC_TEST_POSTGRES_URL must name the owned PostgreSQL fixture")
}

pub fn iggy_url() -> String {
    std::env::var("FRF_CDC_TEST_IGGY_URL")
        .expect("FRF_CDC_TEST_IGGY_URL must name the owned Iggy fixture")
}

pub async fn connect_database() -> Client {
    let (client, connection) = tokio_postgres::connect(&database_url(), NoTls)
        .await
        .expect("connect PostgreSQL fixture");
    tokio::spawn(async move {
        connection.await.expect("PostgreSQL fixture connection");
    });
    client
}

pub async fn install_fixture(client: &Client) {
    client
        .batch_execute(
            "CREATE SCHEMA c008;
             CREATE TABLE c008.orders (
               created_by text NOT NULL,
               tenant_id uuid NOT NULL,
               region text NOT NULL,
               number bigint NOT NULL,
               active boolean,
               quantity integer,
               ratio double precision,
               amount numeric,
               binary_value bytea,
               trace_id uuid,
               occurred_at timestamptz,
               service_day date,
               local_time time,
               local_timestamp timestamp,
               document jsonb,
               description text,
               PRIMARY KEY (region, number)
             );
             ALTER TABLE c008.orders REPLICA IDENTITY FULL;
             CREATE TABLE c008.fixed_records (
               label text NOT NULL,
               record_key text PRIMARY KEY,
               payload text
             );
             CREATE TABLE c008.unsupported_arrays (
               record_key text PRIMARY KEY,
               values integer[] NOT NULL
             );
             CREATE TABLE c008.weak_identity (
               tenant_id uuid NOT NULL,
               label text,
               record_key text PRIMARY KEY
             );
             CREATE PUBLICATION c008_publication FOR TABLE
               c008.orders, c008.fixed_records,
               c008.unsupported_arrays, c008.weak_identity;",
        )
        .await
        .expect("install CDC fixture schema");
}

pub fn orders_enrollment() -> TableEnrollment {
    TableEnrollment {
        schema: "c008".to_owned(),
        table: "orders".to_owned(),
        projection: "summary".to_owned(),
        columns: vec![
            "created_by",
            "tenant_id",
            "region",
            "number",
            "active",
            "quantity",
            "ratio",
            "amount",
            "binary_value",
            "trace_id",
            "occurred_at",
            "service_day",
            "local_time",
            "local_timestamp",
            "document",
            "description",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        tenant: TenantMode::Column {
            column: "tenant_id".to_owned(),
        },
        unsigned_columns: Vec::new(),
    }
}

pub fn fixed_enrollment() -> TableEnrollment {
    TableEnrollment {
        schema: "c008".to_owned(),
        table: "fixed_records".to_owned(),
        projection: "summary".to_owned(),
        columns: vec![
            "label".to_owned(),
            "record_key".to_owned(),
            "payload".to_owned(),
        ],
        tenant: TenantMode::Fixed,
        unsigned_columns: Vec::new(),
    }
}

pub fn config(slot: &str, epoch: &str, enrollments: Vec<TableEnrollment>) -> CdcConfig {
    CdcConfig::new(
        database_url(),
        slot,
        "c008_publication",
        TenantId::from_uuid(TENANT),
        "entity/changes",
    )
    .with_source_epoch(epoch)
    .with_enrollments(enrollments)
}

pub struct RunningConsumer {
    shutdown: watch::Sender<bool>,
    handle: JoinHandle<Result<(), frf_postgres_cdc::consumer::CdcError>>,
}

impl RunningConsumer {
    pub async fn stop(self) -> Result<(), frf_postgres_cdc::consumer::CdcError> {
        self.shutdown.send_replace(true);
        timeout(RECEIVE_TIMEOUT, self.handle)
            .await
            .expect("CDC shutdown timed out")
            .expect("CDC task panicked")
    }

    pub async fn failure(self) -> frf_postgres_cdc::consumer::CdcError {
        timeout(RECEIVE_TIMEOUT, self.handle)
            .await
            .expect("CDC failure timed out")
            .expect("CDC task panicked")
            .expect_err("CDC scenario unexpectedly succeeded")
    }
}

pub async fn start_consumer<L>(config: CdcConfig, broker: Arc<L>) -> RunningConsumer
where
    L: LogBroker + Send + Sync + 'static,
{
    let consumer = frf_postgres_cdc::PostgresCdcConsumer::new(config, broker);
    let (shutdown, shutdown_rx) = watch::channel(false);
    let (ready_tx, mut ready_rx) = watch::channel(false);
    let handle = tokio::spawn(async move {
        let result = consumer
            .run_until_shutdown_with_readiness(shutdown_rx, ready_tx)
            .await;
        if let Err(error) = &result {
            eprintln!("CDC_CONSUMER_ERROR: {error}");
        }
        result
    });
    timeout(RECEIVE_TIMEOUT, async {
        while !*ready_rx.borrow_and_update() {
            ready_rx.changed().await.expect("CDC exited before ready");
        }
    })
    .await
    .expect("CDC readiness timed out");
    RunningConsumer { shutdown, handle }
}

pub async fn receive(stream: &mut EventStream) -> EventEnvelope {
    timeout(RECEIVE_TIMEOUT, stream.next())
        .await
        .expect("timed out waiting for CDC event")
        .expect("CDC broker stream ended")
        .expect("CDC broker stream returned an error")
}

pub async fn subscribe_from<L: LogBroker>(broker: &L, from: Offset) -> EventStream {
    let deadline = tokio::time::Instant::now() + RECEIVE_TIMEOUT;
    loop {
        match broker
            .subscribe(
                ChannelId::WELL_KNOWN_ENTITIES,
                format!("c008-{}", Uuid::new_v4()),
                from,
            )
            .await
        {
            Ok(stream) => return stream,
            Err(error) if tokio::time::Instant::now() < deadline => {
                let _ = error;
                sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("could not subscribe to CDC channel: {error}"),
        }
    }
}

pub async fn confirmed_lsn(client: &Client, slot: &str) -> String {
    client
        .query_one(
            "SELECT confirmed_flush_lsn::text FROM pg_catalog.pg_replication_slots WHERE slot_name = $1",
            &[&slot],
        )
        .await
        .expect("read slot position")
        .get(0)
}

pub async fn assert_lsn_advances(client: &Client, slot: &str, before: &str) {
    let deadline = tokio::time::Instant::now() + RECEIVE_TIMEOUT;
    loop {
        let row = client
            .query_one(
                "SELECT pg_wal_lsn_diff(confirmed_flush_lsn, $2::text::pg_lsn) > 0
                 FROM pg_catalog.pg_replication_slots WHERE slot_name = $1",
                &[&slot, &before],
            )
            .await
            .expect("compare slot position");
        if row.get::<_, bool>(0) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "slot {slot} did not acknowledge its durable commit"
        );
        sleep(Duration::from_millis(100)).await;
    }
}
