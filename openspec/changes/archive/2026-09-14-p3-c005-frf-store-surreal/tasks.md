# Tasks — p3-c005 frf-store-surreal

- [ ] **T1** Create `crates/frf-store-surreal/Cargo.toml`
  - Dependencies: `frf-ports = { path = "../frf-ports" }`, `surrealdb = { workspace = true }`, `bytes = { workspace = true }`, `thiserror = { workspace = true }`, `tracing = { workspace = true }`, `async-trait = { workspace = true }`, `serde = { workspace = true, features = ["derive"] }`, `tokio = { workspace = true, features = ["rt-multi-thread"] }`
  - Features: `integration = []` (gates live-DB tests)
  - `[lints.clippy] pedantic = "warn"`
  - Verification: `cargo check -p frf-store-surreal` exits 0

- [ ] **T2** Create `crates/frf-store-surreal/src/store.rs`
  - Define `SurrealCrdtStore` struct
  - `pub async fn connect(url, ns, db) -> Result<Self, SurrealError>`
    - Connect to SurrealDB WebSocket endpoint
    - Use `db.use_ns(ns).use_db(db).await`
    - Run `DEFINE TABLE crdt_snapshots SCHEMALESS` and index (idempotent)
  - Implement `CrdtStore`:
    - `checkpoint`: `db.upsert("crdt_snapshots").content(...)` with `encoded` as base64 or raw bytes in a BINDATA field
    - `restore`: `db.select("crdt_snapshots:...")` → decode → `CrdtSnapshot`
    - `purge`: `db.delete("crdt_snapshots:...")`
  - `#[tracing::instrument]` on each method
  - No `unwrap()` / `expect()`
  - Verification: `cargo check -p frf-store-surreal` exits 0

- [ ] **T3** Create `crates/frf-store-surreal/src/lib.rs`
  - `pub mod store; pub use store::SurrealCrdtStore;`
  - Verification: `cargo doc -p frf-store-surreal --no-deps` exits 0

- [ ] **T4** Write integration tests in `crates/frf-store-surreal/tests/crdt_store.rs`
  - Gate with `#[cfg(feature = "integration")]`
  - Test `checkpoint_and_restore`: checkpoint a snapshot, restore → same encoded bytes
  - Test `purge`: checkpoint, purge, restore → None
  - Test `overwrite`: checkpoint v1, checkpoint v2 (same entity), restore → v2
  - Verification without live DB: `cargo test -p frf-store-surreal` exits 0 (integration tests skipped); `cargo test -p frf-store-surreal --features integration` requires `SURREAL_URL` env (documented in README)

- [ ] **T5** Create `crates/frf-store-surreal/README.md` (brief)
  - Document: `SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB` env vars needed for integration tests
  - Document: `cargo test --features integration` to run
  - Verification: file exists

- [ ] **T6** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
