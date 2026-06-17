# Tasks — p0-c005 frf-gateway Stub

- [x] **T1** Create `crates/frf-gateway/Cargo.toml`
  - File: `crates/frf-gateway/Cargo.toml`
  - Both `[lib]` (name = "frf_gateway") and `[[bin]]` sections for testability
  - `[dependencies]`: `axum`, `tokio`, `tonic`, `frf-proto`, `frf-domain`, `anyhow`, `tracing`, `tracing-subscriber`, `tower-http`, `serde_json`
  - Add to `[workspace.members]` in root `Cargo.toml`
  - Verification: `cargo check -p frf-gateway` exits 0 ✓

- [x] **T2** Create `crates/frf-gateway/src/main.rs`
  - File: `crates/frf-gateway/src/main.rs`
  - Initializes `tracing_subscriber` with `RUST_LOG` env filter
  - Delegates to `frf_gateway::build_router()` from lib
  - Binds on `0.0.0.0:8080` (configurable via `PORT` env var)
  - Verification: binary builds with `cargo build -p frf-gateway` ✓

- [x] **T3** Create `crates/frf-gateway/src/ws.rs`
  - File: `crates/frf-gateway/src/ws.rs`
  - Axum WS upgrade handler at `/ws`
  - Echoes each received `Message::Text` or `Message::Binary` back to sender
  - Verification: compiles clean ✓

- [x] **T4** Create health route handler
  - File: `crates/frf-gateway/src/routes/health.rs`
  - `pub async fn healthz() -> Json<Value>` returning `{"status":"ok","version":"..."}`
  - Verification: `cargo check -p frf-gateway` exits 0 ✓

- [x] **T5** Create `crates/frf-gateway/tests/health.rs`
  - File: `crates/frf-gateway/tests/health.rs`
  - Uses `axum-test` to spin up the router and assert `GET /healthz` → 200 with `{"status":"ok"}`
  - Verification: `cargo test -p frf-gateway` — test `healthz_returns_200` passes ✓

- [x] **T6** Verify tonic stub compiles
  - `tonic` dependency in Cargo.toml; no active service registered in stub
  - Verification: `cargo build -p frf-gateway` exits 0 with no tonic-related errors ✓
