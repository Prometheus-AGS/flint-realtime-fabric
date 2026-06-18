# p2-c007 — Wire gRPC service into Axum gateway

## Phase
phase-2-generated-sdks

## Depends on
p2-c001 (publish authz fix must be applied first)

## Affected crates
- `crates/frf-gateway/src/lib.rs`
- `crates/frf-gateway/src/main.rs`
- `crates/frf-proto/` (no changes — already generates the `SpineService` server trait)

## What this change does

Phase 1 wired the WebSocket subscription multiplexer. This change adds the
tonic gRPC server surface alongside it, so both Connect-protocol and native
gRPC callers reach the same use-case layer.

### Current state

`frf-gateway/src/lib.rs` serves only the Axum HTTP router (healthz + WS mux).
There is no tonic server, no gRPC route, and no `SpineServiceServer` binding.

### Target state

```
Axum router ─── /healthz (GET)
             ─── /ws     (WebSocket mux)
             ─── /*      (gRPC routes via tonic-into-axum)
```

All three share the same `AppState<L, A, I>`.

### Key implementation notes

1. `tonic` 0.12 exposes `into_router()` on `Server::builder()` that returns an
   `axum::Router` — merge it with the existing Axum router.
2. Implement `tonic::service::grpc::server::Service` (i.e. the generated
   `SpineServiceServer<T>` trait) in a new file
   `crates/frf-gateway/src/grpc_service.rs`.
3. The `grpc_service::SpineGrpcService<L, A, I>` struct holds a clone of
   `AppState<L, A, I>` and delegates to the same use-cases as the WS handler:
   `PublishUseCase` for unary publish, `SubscribePipeline` for server-streaming
   subscribe.
4. `http2_keepalive` must be enabled on the tonic builder for production
   correctness.

### File sketch

```rust
// crates/frf-gateway/src/grpc_service.rs
pub struct SpineGrpcService<L, A, I> { state: AppState<L, A, I> }

#[tonic::async_trait]
impl<L, A, I> SpineService for SpineGrpcService<L, A, I>
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    async fn publish(&self, request: Request<EventEnvelope>) -> Result<Response<PublishResponse>, Status>
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<EventEnvelope, Status>> + Send>>;
    async fn subscribe(&self, request: Request<SubscribeRequest>) -> Result<Response<Self::SubscribeStream>, Status>
}
```

## Exit criteria

- `cargo check --workspace` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
- `cargo test -p frf-gateway` exits 0
- Gateway binary starts and `/healthz` returns 200 (existing behavior preserved)
