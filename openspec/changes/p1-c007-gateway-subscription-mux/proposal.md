# p1-c007 — frf-gateway: Subscription Mux + Publish Endpoint

## Affected crates
- `crates/frf-gateway` (existing — extends the `/healthz` stub from Phase 0)

## Dependency-rule impact
Interface layer. Wires all Phase 1 adapters together: `IggyBroker`, `KetoAuthzProvider`, `OryIdentityVerifier`. Imports `frf-app` use-cases. Composition root — this is the ONLY crate allowed to import concrete adapter crates.

## What this change does

Wires the first live, end-to-end subscription path and a basic publish endpoint into the Axum gateway. This makes Phase 1's exit criterion observable: a WebSocket subscriber can receive events published by another client.

### `AppState`

```rust
pub struct AppState {
    pub subscribe_pipeline: Arc<SubscribePipeline<IggyBroker, KetoAuthzProvider, OryIdentityVerifier>>,
    pub publish_usecase: Arc<PublishUseCase<IggyBroker, KetoAuthzProvider, OryIdentityVerifier>>,
    pub config: Arc<GatewayConfig>,
}
```

Built once at startup in `main.rs`; injected via `axum::extract::State<Arc<AppState>>`.

### New routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/ws/v1/subscribe` | WebSocket upgrade; token in `Authorization: Bearer` header; channel in query param `?channel=<path>` |
| `POST` | `/v1/publish` | Publish one event envelope; JSON body; requires valid JWT |
| `GET` | `/healthz` | Existing — unchanged |

### WebSocket subscribe handler

1. Extract `Authorization: Bearer <token>` from the WS upgrade request headers.
2. Extract `?channel=<path>` query param.
3. Call `SubscribePipeline::execute(SubscribeRequest { token, channel })`.
4. On error: respond with HTTP 401 (identity failure) or 403 (authz denial) before upgrade.
5. On success: upgrade to WebSocket; forward `EventStream` messages as WS text frames (JSON-serialized `EventEnvelope`).
6. On client disconnect: drop the `EventStream`; Iggy consumer stops.

### Publish handler

1. Extract and verify JWT via `IdentityVerifier` (shared from `AppState`).
2. Parse JSON body as `EventEnvelope`.
3. Call `PublishUseCase::execute(envelope)`.
4. Return `200 OK` with `{"offset": <u64>}` or appropriate error status.

### `GatewayConfig`

```rust
pub struct GatewayConfig {
    pub bind_addr: SocketAddr,
    pub iggy_connection_string: String,
    pub keto_base_url: String,
    pub keto_namespace: String,
    pub oathkeeper_jwks_url: String,
    pub jwt_audience: String,
}
```

Loaded from environment variables via `envy` or direct `std::env::var`.

### Module layout changes

```
crates/frf-gateway/src/
├── main.rs          startup, router, AppState construction
├── config.rs        GatewayConfig (new)
├── routes/
│   ├── mod.rs       (new)
│   ├── health.rs    /healthz (moved from main.rs)
│   ├── subscribe.rs /ws/v1/subscribe WebSocket handler (new)
│   └── publish.rs   /v1/publish handler (new)
└── error.rs         GatewayError → axum IntoResponse (new)
```

## Phase 1 exit criterion satisfied

Integration test (`#[ignore]`) starts a local Axum test server (using `axum::Server` or `axum-test`), connects a WebSocket subscriber, publishes one event, and asserts the subscriber receives the event. Requires local Iggy + Keto + Oathkeeper; can be run against test doubles (MockLogBroker, etc.) for CI.

## Non-goals
- Does not implement TLS termination (handled by reverse proxy in production).
- Does not implement `frf-agentproto` AG-UI / A2A message types (Phase 4).
- Does not implement the gRPC Connect-ES transport (Phase 2).
- Does not implement multi-channel fan-out batching optimizations.
