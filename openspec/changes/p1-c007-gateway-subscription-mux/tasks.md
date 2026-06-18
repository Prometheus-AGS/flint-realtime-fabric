# Tasks — p1-c007 frf-gateway subscription mux

- [x] **T1** Create `crates/frf-gateway/src/config.rs`
  - `pub struct GatewayConfig { pub bind_addr: SocketAddr, pub iggy_connection_string: String, pub keto_base_url: String, pub keto_namespace: String, pub oathkeeper_jwks_url: String, pub jwt_audience: String }`
  - `impl GatewayConfig { pub fn from_env() -> anyhow::Result<Self> }` — reads env vars, errors on missing required vars
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T2** Create `crates/frf-gateway/src/error.rs`
  - `pub enum GatewayError { Identity(PortError), Authz(PortError), Publish(AppError), BadRequest(String) }`
  - `impl IntoResponse for GatewayError` — maps to appropriate HTTP status codes (401, 403, 400, 500)
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T3** Create `crates/frf-gateway/src/routes/health.rs`
  - Move existing `/healthz` handler from `main.rs` to this file
  - `pub async fn healthz() -> impl IntoResponse { StatusCode::OK }`
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T4** Create `crates/frf-gateway/src/routes/subscribe.rs`
  - `pub async fn ws_subscribe(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade, headers: HeaderMap, Query(params): Query<SubscribeQuery>) -> impl IntoResponse`
  - Extract Bearer token from `Authorization` header; 401 if missing
  - Extract `channel` from `SubscribeQuery { channel: String }`
  - Call `state.subscribe_pipeline.execute(SubscribeRequest { token, channel })`
  - On error: return `GatewayError::Identity` or `GatewayError::Authz` → HTTP 401/403 before WS upgrade
  - On success: `ws.on_upgrade(move |socket| handle_socket(socket, stream))`
  - `async fn handle_socket(socket: WebSocket, stream: EventStream)` — forward stream items as WS text frames; exit on stream end or send error
  - `#[tracing::instrument(name = "ws::subscribe", skip(state, ws, headers))]`
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T5** Create `crates/frf-gateway/src/routes/publish.rs`
  - `pub async fn publish_event(State(state): State<Arc<AppState>>, headers: HeaderMap, Json(envelope): Json<EventEnvelope>) -> impl IntoResponse`
  - Extract + verify JWT via `state.subscribe_pipeline`'s identity verifier (or extract `IdentityVerifier` from `AppState` directly)
  - Call `state.publish_usecase.execute(envelope)`
  - Return `Json(PublishResponse { offset: u64 })` on success; `GatewayError` on failure
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T6** Update `crates/frf-gateway/src/main.rs`
  - Add `pub struct AppState { subscribe_pipeline: Arc<SubscribePipeline<...>>, publish_usecase: Arc<PublishUseCase<...>>, config: Arc<GatewayConfig> }`
  - In `main()`: load `GatewayConfig::from_env()`, construct adapters (`IggyBroker`, `KetoAuthzProvider`, `OryIdentityVerifier`), build `SubscribePipeline` + `PublishUseCase`, wrap in `Arc<AppState>`
  - Build Axum router: `.route("/healthz", get(health::healthz))` + `.route("/ws/v1/subscribe", get(subscribe::ws_subscribe))` + `.route("/v1/publish", post(publish::publish_event))` + `.with_state(state)`
  - Keep `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T7** Create `crates/frf-gateway/src/routes/mod.rs`
  - `pub mod health; pub mod publish; pub mod subscribe;`
  - Verification: `cargo check -p frf-gateway` exits 0

- [x] **T8** Write integration smoke test
  - File: `crates/frf-gateway/tests/subscribe_mux.rs`
  - Mark with `#[ignore]`
  - Test: bind test server; connect WS subscriber; POST one publish event; assert subscriber receives matching JSON frame
  - Comment: `// Run with: cargo test -p frf-gateway -- --ignored`
  - Uses `tokio-tungstenite` for WS client in test
  - Verification: test compiles; `cargo test -p frf-gateway` (non-ignored) passes

- [x] **T9** Verify full workspace compiles and CI gates pass
  - `cargo check --workspace` exits 0
  - `cargo test --workspace` (non-ignored tests) passes
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
  - `cargo fmt --check --all` exits 0
