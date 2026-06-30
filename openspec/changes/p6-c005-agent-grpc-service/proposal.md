# p6-c005 — AgentGrpcService: RunAgent gRPC Streaming Handler

## Summary

Implement the `AgentService.RunAgent` server-streaming gRPC handler in
`frf-gateway` and wire a dedicated tonic gRPC server on port 9090.

## Open Decision Resolutions

**Proto shape**: Server-streaming only (`request → stream<AgentEvent>`). The
proto definition already uses `returns (stream AgentEvent)`. Bidi is deferred.

**gRPC topology**: Dedicated tonic `Server` on port 9090, separate from Axum on
8080. This avoids HTTP/2 vs HTTP/1.1 mux complexity. Both servers start in
`main.rs` via `tokio::spawn`.

**AppState federation**: Not used in this change. The `AgentGrpcService` takes
`Arc<B>` (agent bus) and `Arc<I>` (identity verifier) directly — no 6th generic
needed for this handler.

## Design

### `domain_to_proto` in `frf-agentproto/src/convert.rs`

Add the inverse conversion (domain → proto) for `AgentEvent`:

```rust
pub fn domain_to_proto(event: frf_domain::AgentEvent) -> fv1::AgentEvent {
    fv1::AgentEvent {
        run_id: event.run_id,
        agent_id: event.agent_id,
        tenant_id: event.tenant_id,
        kind: proto_kind_from_domain(event.kind) as i32,
        content_blocks: event.content_blocks.into_iter().map(block_to_proto).collect(),
        timestamp: event.timestamp,
        protocol: proto_protocol_from_domain(event.protocol) as i32,
    }
}
```

### `AgentGrpcService` (`frf-gateway/src/agent_grpc_service.rs`)

```rust
pub struct AgentGrpcService<B, I> {
    bus: Arc<B>,
    identity: Arc<I>,
}

#[async_trait]
impl<B, I> AgentService for AgentGrpcService<B, I>
where
    B: AgentEventBus + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    type RunAgentStream = Pin<Box<dyn Stream<Item = Result<AgentEvent, Status>> + Send>>;

    async fn run_agent(
        &self,
        request: Request<AgentRunRequest>,
    ) -> Result<Response<Self::RunAgentStream>, Status> {
        // 1. Extract JWT from metadata ("authorization" header)
        // 2. Verify with self.identity.verify_token()
        // 3. Extract tenant_id from VerifiedClaims
        // 4. (Keto subscribe-time check per ADR-002)
        // 5. self.bus.subscribe(&tenant_id) → event stream
        // 6. Map FrfAgentEvent → proto AgentEvent via domain_to_proto
        // 7. Return Response::new(stream)
    }
}
```

### gRPC server in `main.rs`

```rust
// Alongside existing Axum server:
tokio::spawn(async move {
    let grpc_addr = format!("0.0.0.0:{}", config.grpc_port.unwrap_or(9090))
        .parse()
        .expect("invalid gRPC address");
    tonic::Server::builder()
        .add_service(AgentServiceServer::new(agent_grpc_service))
        .serve(grpc_addr)
        .await
        .expect("gRPC server failed");
});
```

### Subscribe-time Keto check (ADR-002 mitigation)

Per ADR-002, add one Keto check at subscribe time in `run_agent`:

```rust
authz.check(subject, "agent_bus:stream", &tenant_id).await
    .map_err(|_| Status::permission_denied("agent bus access denied"))?;
```

This requires `AppState` has `authz: Arc<A>` — the `AgentGrpcService` takes
`Arc<A>` as a fourth field. Or: add it as a constructor parameter. Either
approach avoids a 6th `AppState` generic.

## Files Affected

- `crates/frf-agentproto/src/convert.rs` (MODIFY — add `domain_to_proto`)
- `crates/frf-gateway/src/agent_grpc_service.rs` (NEW)
- `crates/frf-gateway/src/main.rs` (MODIFY — start tonic server on 9090)
- `crates/frf-gateway/src/lib.rs` (MODIFY — expose `AgentGrpcService`)
- `crates/frf-gateway/src/config.rs` (MODIFY — add `grpc_port: Option<u16>`)

## Quality Gates

- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes
- [ ] No `unwrap()` or `expect()` in `agent_grpc_service.rs` (library code)
- [ ] `expect()` in `main.rs` limited to startup — acceptable at binary edge
- [ ] JWT extracted from gRPC metadata `authorization` header (same pattern as WS path)
- [ ] Subscribe-time Keto check present per ADR-002
- [ ] `#[must_use]` on `AgentGrpcService::new()`
