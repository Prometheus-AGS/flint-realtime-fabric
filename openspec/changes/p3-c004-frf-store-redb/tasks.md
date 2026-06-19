# Tasks — p3-c004 frf-store-redb

- [ ] **T1** Create `crates/frf-store-redb/Cargo.toml`
  - Dependencies: `frf-ports = { path = "../frf-ports" }`, `redb = { workspace = true }`, `bytes = { workspace = true }`, `thiserror = { workspace = true }`, `tracing = { workspace = true }`, `async-trait = { workspace = true }`, `tokio = { workspace = true, features = ["rt"] }`
  - Note: redb is synchronous; wrap blocking calls in `tokio::task::spawn_blocking`
  - `[lints.clippy] pedantic = "warn"`
  - Verification: `cargo check -p frf-store-redb` exits 0

- [ ] **T2** Create `crates/frf-store-redb/src/schema.rs`
  - Define redb `TableDefinition<(String, String, u64), Vec<u8>>` named `"pending_ops"`
  - Verification: file compiles

- [ ] **T3** Create `crates/frf-store-redb/src/store.rs`
  - `RedbOpStore { db: Arc<redb::Database> }` with `pub fn open(path: impl AsRef<Path>) -> Result<Self, redb::Error>`
  - Implement `OpStore` for `RedbOpStore`:
    - `queue_op`: `spawn_blocking`, begin write txn, insert `(tenant_id, entity_id, local_seq)` → `payload`; commit
    - `drain_pending`: `spawn_blocking`, begin read txn, range scan `tenant_id + entity_id` prefix, collect into `Vec<PendingOp>`
    - `mark_synced`: `spawn_blocking`, begin write txn, delete range `local_seq <= confirmed_seq`
  - `#[tracing::instrument]` on each method (skip large payload bytes)
  - No `unwrap()` / `expect()` — map `redb::Error` → `PortError::Internal`
  - Verification: `cargo check -p frf-store-redb` exits 0

- [ ] **T4** Create `crates/frf-store-redb/src/lib.rs`
  - `pub mod schema; pub mod store; pub use store::RedbOpStore;`
  - Verification: `cargo doc -p frf-store-redb --no-deps` exits 0

- [ ] **T5** Write integration tests in `crates/frf-store-redb/tests/op_store.rs`
  - Use a `tempfile::tempdir()` redb path
  - Test `queue_and_drain`: queue 3 ops, drain → get 3 in order
  - Test `mark_synced`: queue 3, mark_synced(seq=2), drain → 1 remaining
  - Test `empty_drain`: drain with no ops → empty vec, no error
  - Verification: `cargo test -p frf-store-redb` exits 0; 3 tests pass

- [ ] **T6** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
