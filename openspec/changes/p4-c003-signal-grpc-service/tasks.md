# Tasks — p4-c003 signal-grpc-service

- [ ] **T1** Add `frf-media-livekit` and `tonic-web` to `frf-gateway/Cargo.toml`
  - Add `frf-media-livekit = { path = "../frf-media-livekit" }`
  - Add `tonic-web = "0.14"` (confirm version matches tonic workspace dep)
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T2** Create `SpineSignalService` in `frf-gateway`
  - File: `crates/frf-gateway/src/signal_service.rs` (new file)
  - `pub struct SpineSignalService { media: Arc<dyn MediaSignaling + Send + Sync>, broker: Arc<dyn LogBroker + Send + Sync>, authz: Arc<dyn AuthzProvider + Send + Sync>, identity: Arc<dyn IdentityVerifier + Send + Sync> }`
  - Implement generated `signal_service_server::SignalService` trait:
    - Method `signal`: extract JWT from gRPC metadata, call `identity.verify()`,
      call `authz.check()` for tenant membership, iterate stream with
      `tokio_stream::StreamExt`, call `media.relay_signal(envelope)` per frame,
      publish to spine as `EventKind::Signal`
  - Error on JWT failure: return `tonic::Status::unauthenticated`
  - Error on authz failure: return `tonic::Status::permission_denied`
  - `#[instrument]` on the handler; never log JWT payloads or tenant IDs
  - No unwrap()/expect()
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T3** Extend `AppState` with `media_signaling` field
  - File: `crates/frf-gateway/src/lib.rs` (or wherever `AppState` is defined)
  - Add `media_signaling: Arc<dyn MediaSignaling + Send + Sync>`
  - Verification: `cargo check -p frf-gateway` exits 0; existing routes compile

- [ ] **T4** Mount `SpineSignalService` on the gateway router
  - File: `crates/frf-gateway/src/router.rs` (or `build_router` fn)
  - Add `tonic_web::enable(SignalServiceServer::new(SpineSignalService::from_state(&state)))`
  - Mount at `/flint.v1.SignalService/Signal` via `axum::Router::route_service`
  - Verification: `cargo build -p frf-gateway` exits 0

- [ ] **T5** Construct `LiveKitSignaling` in `main.rs` and inject into `AppState`
  - File: `crates/frf-gateway/src/main.rs`
  - Build `LiveKitSignaling::from_env()` → `Arc<dyn MediaSignaling + Send + Sync>`
  - Pass into `AppState { ..., media_signaling }`
  - Verification: `cargo build -p frf-gateway` exits 0; no unwrap()/expect()

- [ ] **T6** Unit test: `SpineSignalService` rejects missing JWT
  - File: `crates/frf-gateway/src/signal_service.rs` inline `#[cfg(test)]`
  - Mock `IdentityVerifier` returning `Err(PortError::Unauthorized)`, send one
    frame, assert `tonic::Status::UNAUTHENTICATED` returned
  - Verification: `cargo test -p frf-gateway` exits 0
