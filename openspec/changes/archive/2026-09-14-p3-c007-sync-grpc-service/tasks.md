# Tasks — p3-c007 sync-grpc-service

- [ ] **T1** Add `LoroDeltaApplier` to `crates/frf-crdt/src/apply.rs`
  - Implement `ApplyDelta` trait (defined in p3-c006/T1) for a zero-size struct
  - `fn apply(&self, existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, PortError>`
    - Delegates to `crate::merge::apply_delta(existing, delta).map_err(PortError::from)`
  - Export from `frf-crdt::lib.rs` as `pub use apply::LoroDeltaApplier`
  - Verification: `cargo check -p frf-crdt` exits 0

- [ ] **T2** Create `crates/frf-gateway/src/sync_service.rs`
  - `use frf_proto::flint::v1::sync_service_server::SyncService;`
  - `use frf_proto::flint::v1::{SyncRequest, SyncResponse, GetCheckpointRequest, SyncCheckpoint};`
  - Implement `SyncService` for `SyncGrpcService<C, O>`:
    - `sync()`: bidi stream — `tokio_stream::StreamExt` to iterate requests; call `use_case.apply_incoming()` per request; yield `SyncResponse { merged: merged_bytes, ... }`
    - `get_checkpoint()`: `use_case.crdt_store.restore()` → map to `SyncCheckpoint`
  - JWT extraction: replicate the pattern from `SpineGrpcService` — extract `Authorization: Bearer <token>` from tonic metadata; call `identity_verifier.verify()`. Return `tonic::Status::unauthenticated` if missing or invalid.
  - `#[tracing::instrument(skip(self, request))]` on each method
  - No `unwrap()` / `expect()`
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T3** Register `SyncServiceServer` in `crates/frf-gateway/src/main.rs`
  - Instantiate `SurrealCrdtStore::connect(...)` from env vars (`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`)
  - Instantiate `RedbOpStore::open(op_log_path)` from env var (`REDB_PATH`, default `./op_log.redb`)
  - Instantiate `LoroDeltaApplier`
  - Instantiate `SyncUseCase::new(surreal_store, redb_store, apply_delta)`
  - Instantiate `SyncGrpcService::new(sync_use_case)`
  - Add `.add_service(SyncServiceServer::new(sync_service))` to router
  - Verification: `cargo build -p frf-gateway` exits 0

- [ ] **T4** Add gateway Cargo.toml deps
  - Add `frf-crdt = { path = "../frf-crdt" }`, `frf-store-redb = { path = "../frf-store-redb" }`, `frf-store-surreal = { path = "../frf-store-surreal" }` to `crates/frf-gateway/Cargo.toml`
  - Verify dependency rule: these are in `[dependencies]` of gateway only (not domain/app/ports)
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T5** Verify gateway `/healthz` still returns 200 after changes
  - `cargo run -p frf-gateway &` (background), wait 2s
  - `curl -s -o /dev/null -w "%{http_code}" http://localhost:4000/healthz` → `200`
  - Kill background gateway
  - Verification: exit code 0 from curl check

- [ ] **T6** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
