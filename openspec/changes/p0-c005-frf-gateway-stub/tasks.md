# Tasks — p0-c005 frf-gateway Stub

- [ ] **T1** Create `crates/frf-gateway/Cargo.toml`
  - File: `crates/frf-gateway/Cargo.toml`
  - Binary crate (no `[lib]`)
  - `[dependencies]`: `axum = { workspace = true, features = ["ws"] }`, `tokio = { workspace = true, features = ["full"] }`, `tonic = { workspace = true }`, `frf-proto = { path = "../frf-proto" }`, `frf-domain = { path = "../frf-domain" }`, `anyhow = { workspace = true }`, `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true, features = ["env-filter"] }`
  - Add to `[workspace.members]` in root `Cargo.toml`
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T2** Create `crates/frf-gateway/src/main.rs`
  - File: `crates/frf-gateway/src/main.rs`
  - Initializes `tracing_subscriber` with `RUST_LOG` env filter
  - Builds Axum router: `GET /healthz` → `{"status":"ok","version":"0.1.0"}`
  - Binds on `0.0.0.0:8080` (configurable via `PORT` env var)
  - Calls `ws::mount_ws_echo(&mut router)` for WS handler
  - ≤ 150 lines; WS handler extracted to `src/ws.rs`
  - Verification: binary builds with `cargo build -p frf-gateway`

- [ ] **T3** Create `crates/frf-gateway/src/ws.rs`
  - File: `crates/frf-gateway/src/ws.rs`
  - Axum WS upgrade handler at `/ws`
  - Echoes each received `Message::Text` or `Message::Binary` back to sender
  - No business logic; no frf-ports wiring yet
  - Verification: unit test in same file verifies frame echo behavior

- [ ] **T4** Create health route handler
  - File: `crates/frf-gateway/src/routes/health.rs`
  - `async fn healthz() -> impl IntoResponse` returning JSON `{"status":"ok"}`
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T5** Create `crates/frf-gateway/tests/health.rs`
  - File: `crates/frf-gateway/tests/health.rs`
  - Uses `axum_test` or `hyper` to spin up the router and assert `GET /healthz` → 200 with `{"status":"ok"}`
  - No external network calls
  - Verification: `cargo test -p frf-gateway` all tests pass

- [ ] **T6** Verify tonic stub compiles
  - No real service registered — just `tonic::transport::Server::builder()` bound in main (or no-op service)
  - Verification: `cargo build -p frf-gateway` exits 0 with no tonic-related errors
