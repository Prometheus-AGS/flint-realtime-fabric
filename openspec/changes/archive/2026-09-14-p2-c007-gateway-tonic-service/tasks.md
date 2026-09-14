# Tasks — p2-c007 gateway-tonic-service

- [ ] **T1** Add tonic server dependency to `crates/frf-gateway/Cargo.toml`
  - Add `tonic = { workspace = true, features = ["transport"] }` (already in workspace)
  - Verify `frf-proto` is already listed as a dependency (it is from Phase 1)
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T2** Create `crates/frf-gateway/src/grpc_service.rs`
  - Define `SpineGrpcService<L, A, I>` struct holding `AppState<L, A, I>`
  - Implement `frf_proto::spine_service_server::SpineService` for `SpineGrpcService<L, A, I>`
    - `publish`: extract token from gRPC metadata, call `PublishUseCase::execute`, return `PublishResponse`
    - `subscribe`: call `SubscribePipeline::execute`, stream `EventEnvelope` items
  - Bounds: `L: LogBroker + Send + Sync + 'static`, `A: AuthzProvider + Send + Sync + 'static`, `I: IdentityVerifier + Send + Sync + 'static`
  - No `unwrap()`; map errors to `tonic::Status::internal` / `Status::unauthenticated`
  - Add `#[tracing::instrument]` spans on both methods
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T3** Wire tonic into Axum router in `crates/frf-gateway/src/lib.rs`
  - Build `SpineServiceServer::new(SpineGrpcService { state: state.clone() })`
  - Build tonic router: `tonic::transport::Server::builder().add_service(spine_svc).into_router()`
  - Merge with existing Axum router: `axum_router.merge(grpc_router)`
  - Enable `http2_keepalive_interval` on the tonic builder
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T4** Update `crates/frf-gateway/src/main.rs` if needed
  - Ensure `serve` call uses `axum::Server::from_tcp` with `tcp_nodelay(true)` for gRPC
  - Verify no duplicate listener binding
  - Verification: `cargo build -p frf-gateway` exits 0

- [ ] **T5** Add unit tests in `crates/frf-gateway/src/grpc_service.rs`
  - `test_publish_unauthenticated` — no metadata token → `Status::unauthenticated`
  - `test_publish_ok` — mock `IdentityVerifier` + mock `AuthzProvider` + mock `LogBroker` → returns `PublishResponse`
  - Mirror the pattern used in `crates/frf-gateway/src/handlers/` tests
  - Verification: `cargo test -p frf-gateway` exits 0

- [ ] **T6** Clippy gate
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
  - Verification: exits 0
