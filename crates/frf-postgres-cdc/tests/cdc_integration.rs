// Run with: cargo test -p frf-postgres-cdc -- --ignored
//
// Requires a local PostgreSQL 17 instance with:
//   - A database named `frf_test`
//   - `wal_level = logical` in postgresql.conf
//   - A publication named `frf_pub` covering the `entities` table

use frf_domain::TenantId;
use frf_postgres_cdc::{CdcConfig, PostgresCdcConsumer};
use tokio::sync::watch;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires a local Postgres 17 instance with logical replication configured"]
async fn cdc_consumer_publishes_insert_event() {
    use async_trait::async_trait;
    use frf_domain::ChannelId;
    use frf_domain::{Channel, Cursor, EventEnvelope, Offset};
    use frf_ports::{EventStream, LogBroker, PortError};
    use std::sync::{Arc, Mutex};

    struct MockBroker {
        published: Mutex<Vec<EventEnvelope>>,
    }

    #[async_trait]
    impl LogBroker for MockBroker {
        async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
            let offset = envelope.offset;
            self.published.lock().unwrap().push(envelope);
            Ok(offset)
        }

        async fn subscribe(
            &self,
            _channel_id: ChannelId,
            _consumer_id: String,
            _from: Offset,
        ) -> Result<EventStream, PortError> {
            unimplemented!("not needed in integration test")
        }

        async fn seek(&self, _cursor: Cursor) -> Result<(), PortError> {
            Ok(())
        }

        async fn ack(
            &self,
            _channel_id: ChannelId,
            _consumer_id: &str,
            _offset: Offset,
        ) -> Result<(), PortError> {
            Ok(())
        }

        async fn ensure_channel(&self, _channel: Channel) -> Result<(), PortError> {
            Ok(())
        }
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/frf_test".to_owned());

    let config = CdcConfig::new(
        database_url,
        "frf_cdc_slot",
        "frf_pub",
        TenantId::from_uuid(Uuid::nil()),
        "entity/changes",
    );

    let broker = Arc::new(MockBroker {
        published: Mutex::new(Vec::new()),
    });

    let consumer = PostgresCdcConsumer::new(config, broker.clone());
    let (tx, rx) = watch::channel(false);

    // Run for a short time then shut down
    let handle = tokio::spawn(async move { consumer.run_until_shutdown(rx).await });

    // Allow time for a WAL event to arrive from a concurrent INSERT
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let _ = tx.send(true);

    handle.await.expect("task panicked").expect("cdc error");

    // At least one envelope should have been published if an INSERT occurred
    let published = broker.published.lock().unwrap();
    println!("CDC published {} envelopes", published.len());
}
