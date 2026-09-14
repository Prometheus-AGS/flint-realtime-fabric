# Tasks — p5-c004-gateway-agent-service

- [ ] Add `frf-librefang` and `frf-agentproto` to `crates/frf-gateway/Cargo.toml`
- [ ] Add `B: AgentEventBus` generic to `AppState` struct in `crates/frf-gateway/src/lib.rs`
- [ ] Update `build_router` signature and bounds to include `B: AgentEventBus`
- [ ] Add `/ws/v1/agents` route to `build_router`
- [ ] Create `crates/frf-gateway/src/routes/agents.rs` with `ws_agent_stream` handler
- [ ] Add `agents` module to `crates/frf-gateway/src/routes/mod.rs`
- [ ] Create `crates/frf-gateway/src/agent_grpc_service.rs` with `AgentServiceImpl<B>`
- [ ] Wire `LibreFangBus::start().await` into `AppState` in `crates/frf-gateway/src/main.rs`
- [ ] Expose `AgentServiceImpl` via tonic gRPC server in `main.rs`
- [ ] Security: extract `tenant_id` from `VerifiedClaims`, not URL params
- [ ] `cargo check --workspace` clean
- [ ] `cargo clippy --workspace -- -D warnings -W clippy::pedantic` clean
- [ ] Integration test: WS upgrade to `/ws/v1/agents` returns 101
