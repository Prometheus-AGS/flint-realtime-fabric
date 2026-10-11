#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Gateway composition acceptance using live PostgreSQL 17 and durable Iggy.

#[path = "support/cdc/mod.rs"]
mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset, TenantId};
use frf_ports::{EventStream, LogBroker, PortError};
use frf_postgres_cdc::consumer::CdcError;
use frf_postgres_cdc::model::{CanonicalValue, CdcMutation};
use frf_postgres_cdc::{TableEnrollment, TenantMode};
use tokio::time::timeout;

use support::{
    COLUMN_TENANT, TENANT, assert_lsn_advances, config, confirmed_lsn, connect_database,
    fixed_enrollment, install_fixture, orders_enrollment, receive, start_consumer, subscribe_from,
};

struct FailAfterDurablePublish {
    inner: Arc<IggyBroker>,
    fail_once: AtomicBool,
}

impl FailAfterDurablePublish {
    fn new(inner: Arc<IggyBroker>) -> Self {
        Self {
            inner,
            fail_once: AtomicBool::new(true),
        }
    }
}

#[async_trait]
impl LogBroker for FailAfterDurablePublish {
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        let offset = self.inner.publish(envelope).await?;
        if self.fail_once.swap(false, Ordering::SeqCst) {
            return Err(PortError::Transport(
                "injected crash after durable publication".to_owned(),
            ));
        }
        Ok(offset)
    }

    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError> {
        self.inner.subscribe(channel_id, consumer_id, from).await
    }

    async fn seek(&self, cursor: Cursor) -> Result<(), PortError> {
        self.inner.seek(cursor).await
    }

    async fn ack(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
        offset: Offset,
    ) -> Result<(), PortError> {
        self.inner.ack(channel_id, consumer_id, offset).await
    }

    async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError> {
        self.inner.ensure_channel(channel).await
    }
}

fn decode(envelope: EventEnvelope) -> CdcMutation {
    serde_json::from_value(envelope.payload).expect("decode CDC mutation")
}

async fn assert_quiet(stream: &mut EventStream) {
    assert!(
        timeout(
            Duration::from_secs(2),
            futures_util::StreamExt::next(stream),
        )
        .await
        .is_err(),
        "unexpected CDC event advanced the broker"
    );
}

async fn assert_invalid_enrollment(
    broker: Arc<IggyBroker>,
    enrollment: TableEnrollment,
    expected: &str,
) {
    let consumer = frf_postgres_cdc::PostgresCdcConsumer::new(
        config("c008_invalid", "c008-invalid", vec![enrollment]),
        broker,
    );
    let (_shutdown, shutdown_rx) = tokio::sync::watch::channel(false);
    let error = consumer
        .run_until_shutdown(shutdown_rx)
        .await
        .expect_err("invalid enrollment must fail before streaming");
    assert!(
        error.to_string().contains(expected),
        "unexpected error: {error}"
    );
}

#[tokio::test]
#[ignore = "requires the owned PostgreSQL and Iggy CDC fixture"]
#[allow(clippy::too_many_lines)]
async fn cdc_commit_mapping_contract() {
    let client = connect_database().await;
    install_fixture(&client).await;
    let broker = Arc::new(
        IggyBroker::new(&support::iggy_url())
            .await
            .expect("connect owned Iggy fixture"),
    );
    broker
        .ensure_channel(Channel {
            id: ChannelId::WELL_KNOWN_ENTITIES,
            tenant_id: TenantId::from_uuid(TENANT),
            path: "entity/changes".to_owned(),
        })
        .await
        .expect("create CDC channel");

    let core_config = config(
        "c008_core",
        "c008-core-v1",
        vec![orders_enrollment(), fixed_enrollment()],
    );
    let core = start_consumer(core_config, Arc::clone(&broker)).await;
    let core_before = confirmed_lsn(&client, "c008_core").await;
    client
        .batch_execute(
            "BEGIN;
             INSERT INTO c008.orders VALUES (
               'alice', '00000000-0000-0000-0000-000000000008', 'us', 41,
               true, -7, 1.5, 100.2500, decode('00ff', 'hex'),
               'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', '2026-09-15T12:34:56.123456Z',
               '2026-09-15', '12:34:56.123456', '2026-09-15 12:34:56.123456',
               '{\"z\":1,\"a\":[true,null]}',
               (SELECT string_agg(md5(g::text), '') FROM generate_series(1, 4000) g)
             );
             INSERT INTO c008.orders VALUES (
               'bob', '00000000-0000-0000-0000-000000000008', 'eu', 42,
               false, 9, -2.25, -0.500, decode('1020', 'hex'),
               'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', '2026-09-15T13:00:00Z',
               '2026-09-16', '01:02:03', '2026-09-16 01:02:03',
               '{\"b\":2,\"a\":1}', 'short'
             );
             INSERT INTO c008.fixed_records(label, record_key, payload)
               VALUES ('fixed', 'text-key', 'value');
             COMMIT;
             BEGIN;
             INSERT INTO c008.orders(created_by, tenant_id, region, number)
               VALUES ('rollback', '00000000-0000-0000-0000-000000000008', 'rolled-back', 99);
             ROLLBACK;
             UPDATE c008.orders SET amount = 101.500 WHERE region = 'us' AND number = 41;
             DELETE FROM c008.orders WHERE region = 'eu' AND number = 42;
             DELETE FROM c008.fixed_records WHERE record_key = 'text-key';",
        )
        .await
        .expect("write core CDC transactions");

    let mut stream = subscribe_from(broker.as_ref(), Offset::BEGINNING).await;
    let mut envelopes = Vec::new();
    for _ in 0..6 {
        envelopes.push(receive(&mut stream).await);
    }
    assert!(
        envelopes
            .windows(2)
            .all(|pair| pair[1].offset == pair[0].offset.next()),
        "broker positions must be unique and contiguous within multi-row commits"
    );
    assert_quiet(&mut stream).await;
    let core_last = envelopes.last().expect("six core envelopes").offset;
    let mutations: Vec<CdcMutation> = envelopes.iter().cloned().map(decode).collect();
    assert!(
        envelopes
            .iter()
            .zip(&mutations)
            .all(|(envelope, mutation)| envelope.channel.tenant_id == mutation.tenant_id),
        "envelope and source payload tenants must match"
    );
    assert_eq!(
        mutations[0].source.commit_lsn,
        mutations[1].source.commit_lsn
    );
    assert_eq!(
        mutations[1].source.commit_lsn,
        mutations[2].source.commit_lsn
    );
    assert_eq!(mutations[0].source.transaction_index, 0);
    assert_eq!(mutations[1].source.transaction_index, 1);
    assert_eq!(mutations[2].source.transaction_index, 2);
    assert_eq!(mutations[0].entity_type, "c008.orders@summary");
    assert_eq!(mutations[0].key.parts[0].column, "region");
    assert_eq!(mutations[0].key.parts[1].column, "number");
    assert_eq!(*mutations[0].tenant_id.as_uuid(), COLUMN_TENANT);
    assert_eq!(mutations[2].entity_type, "c008.fixed_records@summary");
    assert_eq!(mutations[2].key.parts[0].column, "record_key");
    assert_eq!(*mutations[2].tenant_id.as_uuid(), TENANT);
    assert!(
        mutations[0]
            .record
            .as_ref()
            .expect("insert record")
            .iter()
            .any(|field| matches!(field.value, CanonicalValue::Json(_)))
    );
    assert!(
        mutations[0]
            .record
            .as_ref()
            .expect("insert record")
            .iter()
            .any(|field| matches!(field.value, CanonicalValue::TimestampWithoutZone(_)))
    );
    assert_eq!(mutations[3].unchanged_toast, vec!["description"]);
    assert!(mutations[4].record.is_none());
    assert!(mutations[5].record.is_none());
    core.stop().await.expect("stop core consumer");
    assert_lsn_advances(&client, "c008_core", &core_before).await;

    let crash_config = config(
        "c008_crash",
        "c008-crash-v1",
        vec![orders_enrollment(), fixed_enrollment()],
    );
    let crash_broker = Arc::new(FailAfterDurablePublish::new(Arc::clone(&broker)));
    let crash = start_consumer(crash_config.clone(), crash_broker).await;
    let crash_before = confirmed_lsn(&client, "c008_crash").await;
    client
        .batch_execute(
            "BEGIN;
             INSERT INTO c008.fixed_records(label, record_key) VALUES ('crash-a', 'crash-a');
             INSERT INTO c008.fixed_records(label, record_key) VALUES ('crash-b', 'crash-b');
             COMMIT;",
        )
        .await
        .expect("write crash-window transaction");
    assert!(matches!(crash.failure().await, CdcError::Broker(_)));
    assert_eq!(confirmed_lsn(&client, "c008_crash").await, crash_before);
    let mut crash_stream = subscribe_from(broker.as_ref(), core_last.next()).await;
    let first_attempt = receive(&mut crash_stream).await;
    assert_eq!(
        decode(first_attempt.clone()).key.parts[0].column,
        "record_key"
    );

    let restarted = start_consumer(crash_config, Arc::clone(&broker)).await;
    let second_attempt = receive(&mut crash_stream).await;
    assert_eq!(second_attempt.offset, first_attempt.offset.next());
    assert_eq!(decode(second_attempt.clone()).source.transaction_index, 1);
    assert_quiet(&mut crash_stream).await;
    restarted.stop().await.expect("stop restarted CDC consumer");
    assert_lsn_advances(&client, "c008_crash", &crash_before).await;

    let poison_config = config(
        "c008_poison",
        "c008-poison-v1",
        vec![orders_enrollment(), fixed_enrollment()],
    );
    let poison = start_consumer(poison_config, Arc::clone(&broker)).await;
    let poison_before = confirmed_lsn(&client, "c008_poison").await;
    client
        .batch_execute(
            "INSERT INTO c008.orders(created_by, tenant_id, region, number, amount)
             VALUES ('poison', '00000000-0000-0000-0000-000000000008', 'poison', 1, 'NaN');",
        )
        .await
        .expect("write poison transaction");
    assert!(matches!(poison.failure().await, CdcError::Transaction(_)));
    assert_eq!(confirmed_lsn(&client, "c008_poison").await, poison_before);
    let mut poison_probe = subscribe_from(broker.as_ref(), second_attempt.offset.next()).await;
    assert_quiet(&mut poison_probe).await;

    let unsupported = TableEnrollment {
        schema: "c008".to_owned(),
        table: "unsupported_arrays".to_owned(),
        projection: "summary".to_owned(),
        columns: vec!["record_key".to_owned(), "values".to_owned()],
        tenant: TenantMode::Fixed,
        unsigned_columns: Vec::new(),
    };
    assert_invalid_enrollment(
        Arc::clone(&broker),
        unsupported,
        "unsupported PostgreSQL type",
    )
    .await;
    let weak = TableEnrollment {
        schema: "c008".to_owned(),
        table: "weak_identity".to_owned(),
        projection: "summary".to_owned(),
        columns: vec!["record_key".to_owned(), "label".to_owned()],
        tenant: TenantMode::Column {
            column: "tenant_id".to_owned(),
        },
        unsigned_columns: Vec::new(),
    };
    assert_invalid_enrollment(Arc::clone(&broker), weak, "inadequate replica identity").await;

    let drift_config = config(
        "c008_drift",
        "c008-drift-v1",
        vec![orders_enrollment(), fixed_enrollment()],
    );
    let drift = start_consumer(drift_config, Arc::clone(&broker)).await;
    let drift_before = confirmed_lsn(&client, "c008_drift").await;
    client
        .batch_execute(
            "ALTER TABLE c008.orders ADD COLUMN surprise text;
             UPDATE c008.orders SET active = false WHERE region = 'us' AND number = 41;",
        )
        .await
        .expect("write schema-drift transaction");
    assert!(matches!(drift.failure().await, CdcError::Enrollment(_)));
    assert_eq!(confirmed_lsn(&client, "c008_drift").await, drift_before);
    assert_quiet(&mut poison_probe).await;

    println!(
        "CDC_COMMIT_MAPPING_PASS core=6 crash_unique=2 poison_blocked=true schema_blocked=true"
    );
}
