# Tasks — p5-c003-frf-librefang

- [ ] Resolve BossFang fork URL (git dep or crates.io) — document in ADR or comment
- [ ] Add `frf-librefang` to workspace members in root `Cargo.toml`
- [ ] Add `bossfang` (or equivalent) as workspace dep
- [ ] Create `crates/frf-librefang/Cargo.toml`
- [ ] Create `crates/frf-librefang/src/error.rs` with `LibreFangError` (thiserror)
- [ ] Create `crates/frf-librefang/src/supervisor.rs` with `LibreFangSupervisor` ractor actor
- [ ] Create `crates/frf-librefang/src/publisher.rs` with `PublisherActor` that broadcasts to subscribers
- [ ] Create `crates/frf-librefang/src/subscriber.rs` with `TenantSubscriber` actor holding mpsc sender
- [ ] Create `crates/frf-librefang/src/bus.rs` with `LibreFangBus` + `impl AgentEventBus`
- [ ] Create `crates/frf-librefang/src/lib.rs` with re-exports
- [ ] Create `crates/frf-librefang/tests/bus_smoke.rs`: start bus, publish event, receive from subscriber stream
- [ ] `cargo test -p frf-librefang` passes
- [ ] `cargo clippy -p frf-librefang -- -D warnings -W clippy::pedantic` clean
