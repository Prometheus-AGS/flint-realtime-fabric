# p5-c004 — Gateway AgentService + AppState actor bus wiring

## Summary

Add a 5th generic `B: AgentEventBus` to `AppState`, wire `LibreFangBus` in
`main.rs`, and implement the `AgentService` gRPC handler + a WebSocket
agent-event fan-out route.

## Motivation

The gateway is the composition root. Once `frf-librefang` provides a concrete
`AgentEventBus`, this change wires it into the gateway so that:
1. gRPC `RunAgent` calls flow into the bus
2. WebSocket subscribers receive fan-out agent events

## Design

### AppState expansion

```rust
pub struct AppState<L, A, I, M, B> {
    pub subscribe_pipeline: Arc<SubscribePipeline<L, A, I>>,
    pub publish_usecase: Arc<PublishUseCase<L, A, I>>,
    pub media_signaler: Arc<M>,
    pub agent_bus: Arc<B>,
    pub config: Arc<GatewayConfig>,
}
```

`build_router` gains the `B: AgentEventBus` bound and a new route:
```rust
.route("/ws/v1/agents", get(routes::agents::ws_agent_stream::<L, A, I, M, B>))
```

### AgentServiceImpl (gRPC)

```rust
// frf-gateway/src/agent_grpc_service.rs

pub struct AgentServiceImpl<B> {
    bus: Arc<B>,
}

#[tonic::async_trait]
impl<B: AgentEventBus> AgentService for AgentServiceImpl<B> {
    type RunAgentStream = /* tokio_stream::wrappers::ReceiverStream<...> */;

    async fn run_agent(
        &self,
        request: Request<AgentRunRequest>,
    ) -> Result<Response<Self::RunAgentStream>, Status> {
        let req = request.into_inner();
        // 1. Subscribe to tenant stream BEFORE publishing RunStart
        let stream = self.bus.subscribe(&req.tenant_id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        // 2. Publish RunStart event
        self.bus.publish(/* AgentEvent::run_start from req */).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(stream.map(|e| proto_from_domain(e))))
    }
}
```

### WS agent stream route

```rust
// frf-gateway/src/routes/agents.rs

pub async fn ws_agent_stream<L, A, I, M, B>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<L, A, I, M, B>>>,
    // JWT claims from Oathkeeper header (per CLAUDE.md security constraints)
    claims: VerifiedClaims,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent_ws(socket, state, claims))
}
```

**Security**: JWT claims are verified at this boundary per CLAUDE.md — tenant_id
extracted from claims, not from query params. Keto `check` is not required for
subscribe-to-own-tenant streams (visibility gate for agent events is tenant equality,
not fine-grained Zanzibar lookup — document this in ADR if departing from entity pattern).

## Files Changed

- `crates/frf-gateway/src/lib.rs` — add `B` generic to `AppState` and `build_router`
- `crates/frf-gateway/src/main.rs` — wire `LibreFangBus::start().await` as `B` in `AppState`
- `crates/frf-gateway/src/agent_grpc_service.rs` — NEW `AgentServiceImpl`
- `crates/frf-gateway/src/routes/agents.rs` — NEW WS agent fan-out route
- `crates/frf-gateway/Cargo.toml` — add `frf-librefang`, `frf-agentproto` deps
- `crates/frf-gateway/src/routes/mod.rs` — expose `agents` module

## Acceptance Criteria

- [ ] `cargo check --workspace` clean after adding 5th generic
- [ ] `AppState<L, A, I, M, B>` compiles with `B: AgentEventBus`
- [ ] `/ws/v1/agents` route responds to WS upgrade (integration test)
- [ ] gRPC `RunAgent` returns a streaming response (test with tonic test client)
- [ ] Tenant extracted from JWT claims, not query string (security constraint)
- [ ] No `unwrap()` in gateway code
- [ ] `clippy::pedantic` passes
