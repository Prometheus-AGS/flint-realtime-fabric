# Tasks — p1-c006 frf-postgres-cdc

- [x] **T1** Create `crates/frf-postgres-cdc/src/config.rs`
  - `pub struct CdcConfig { pub database_url: String, pub slot_name: String, pub publication_name: String, pub tenant_id: TenantId, pub channel_path: String, pub lsn_checkpoint_interval: u64 }`
  - `impl Default for CdcConfig` — sensible defaults for `lsn_checkpoint_interval` (1000)
  - Verification: `cargo check -p frf-postgres-cdc` exits 0

- [x] **T2** Create `crates/frf-postgres-cdc/src/decode.rs`
  - `decode_insert(relation: &Relation, row: &[Column]) -> Result<EntityChange, DecodeError>`
  - `decode_update(relation: &Relation, old_row: Option<&[Column]>, new_row: &[Column]) -> Result<EntityChange, DecodeError>`
  - `decode_delete(relation: &Relation, old_row: &[Column]) -> Result<EntityChange, DecodeError>`
  - WAL → `EntityChange` mapping: `entity_type` from relation name, `entity_id` from primary key column, `data` as `serde_json::Value`, `op` from `ChangeOp` enum
  - Unit tests in `#[cfg(test)]` block with manually constructed WAL structures
  - Verification: `cargo test -p frf-postgres-cdc -- decode` passes

- [x] **T3** Create `crates/frf-postgres-cdc/src/consumer.rs`
  - `pub struct PostgresCdcConsumer<L: LogBroker> { config: CdcConfig, broker: Arc<L> }`
  - `pub async fn run_until_shutdown(&self, shutdown: tokio::sync::watch::Receiver<bool>) -> anyhow::Result<()`
    - Connect to Postgres via `tokio_postgres::connect`
    - Issue `CREATE_REPLICATION_SLOT` if not exists (catch AlreadyExists)
    - Issue `START_REPLICATION SLOT ... LOGICAL ... PLUGIN pgoutput`
    - Decode WAL messages in loop; publish `EventEnvelope` via `broker.publish()`
    - Send `StandbyStatusUpdate` every `lsn_checkpoint_interval` messages
    - Exit cleanly when `shutdown` channel fires
  - `#[tracing::instrument(name = "cdc::run", skip(self, shutdown))]`
  - Verification: `cargo check -p frf-postgres-cdc` exits 0

- [x] **T4** Update `crates/frf-postgres-cdc/src/lib.rs`
  - `pub mod config; pub mod consumer; pub mod decode;`
  - `pub use config::CdcConfig; pub use consumer::PostgresCdcConsumer;`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-postgres-cdc` exits 0

- [x] **T5** Write unit tests for WAL decode
  - Covers: insert, update (with old row), update (without old row), delete
  - Uses synthetic `Relation` + `Column` structs (no real Postgres connection)
  - Verification: `cargo test -p frf-postgres-cdc` — unit tests pass

- [x] **T6** Write integration test (requires local Postgres)
  - File: `crates/frf-postgres-cdc/tests/cdc_integration.rs`
  - Mark with `#[ignore]`
  - Test: connect to test Postgres, create replication slot, insert a row, assert `EventEnvelope` published to a `MockLogBroker`
  - Comment: `// Run with: cargo test -p frf-postgres-cdc -- --ignored`
  - Verification: test compiles; manually runnable against local Postgres 17
