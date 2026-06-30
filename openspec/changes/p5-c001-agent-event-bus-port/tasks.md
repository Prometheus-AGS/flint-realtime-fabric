# Tasks — p5-c001-agent-event-bus-port

- [ ] Add `async-trait` and `futures` to `frf-ports/Cargo.toml` workspace dep (if not present)
- [ ] Create `crates/frf-ports/src/agent_bus.rs` with `AgentEventBus` trait + `AgentEventStream` type alias
- [ ] Add `pub mod agent_bus` to `crates/frf-ports/src/lib.rs`
- [ ] Re-export `AgentEventBus` and `AgentEventStream` from `frf-ports` crate root
- [ ] `cargo check -p frf-ports` clean
- [ ] `cargo clippy -p frf-ports -- -D warnings -W clippy::pedantic` clean
