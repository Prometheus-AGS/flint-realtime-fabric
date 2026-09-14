# p5-c003 — frf-librefang crate (BossFang actor bus)

## Summary

Create the `frf-librefang` crate that wraps BossFang (GQAdonis fork of
LibreFang) and implements the `AgentEventBus` port trait using ractor actors.

## Motivation

BossFang is the project's spine actor bus for agent events. Rather than
calling Iggy directly for agent-protocol events, `frf-librefang` provides
a ractor-based publish/subscribe model where the gateway supervisor tree
manages actor lifecycle and fan-out to WebSocket consumers.

## Open Decision (must resolve before implement)

**BossFang fork source**: Either:
- `{ git = "https://github.com/GQAdonis/librefang", branch = "main" }` — use git dep
- Published to crates.io as `bossfang = "x.y.z"` — use version dep

The crate name used in this proposal is `bossfang` (git dep). Update if crates.io path differs.

## Design

### Actor Topology

```
LibreFangSupervisor (ractor::SupervisionTree)
├── PublisherActor   — receives publish() calls, broadcasts to subscribers
└── SubscriberRegistry — manages per-tenant subscriber actors
    ├── TenantSubscriber(tenant_id: "a") — streams events to WS handler
    └── TenantSubscriber(tenant_id: "b")
```

### Port Implementation

```rust
// frf-librefang/src/bus.rs

pub struct LibreFangBus {
    supervisor: ActorRef<SupervisorMessage>,
}

#[async_trait]
impl AgentEventBus for LibreFangBus {
    async fn publish(&self, event: AgentEvent) -> Result<(), PortError> {
        self.supervisor
            .send_message(SupervisorMessage::Publish(event))
            .map_err(|e| PortError::Unavailable(e.to_string()))
    }

    async fn subscribe(&self, tenant_id: &str) -> Result<AgentEventStream, PortError> {
        // Send Subscribe message, receive a tokio::sync::mpsc::Receiver back
        // Convert to Stream
    }
}
```

### Start-up

```rust
impl LibreFangBus {
    pub async fn start() -> Result<Self, LibreFangError> {
        let (supervisor, _handle) =
            Actor::spawn(None, LibreFangSupervisor, ()).await
                .map_err(LibreFangError::SpawnFailed)?;
        Ok(Self { supervisor })
    }
}
```

## Files Changed

- `crates/frf-librefang/Cargo.toml` — NEW crate; deps: ractor, frf-domain, frf-ports, frf-agentproto, tokio, async-trait, thiserror
- `crates/frf-librefang/src/lib.rs`
- `crates/frf-librefang/src/bus.rs` — `LibreFangBus` + `AgentEventBus` impl
- `crates/frf-librefang/src/supervisor.rs` — `LibreFangSupervisor` actor
- `crates/frf-librefang/src/publisher.rs` — `PublisherActor`
- `crates/frf-librefang/src/subscriber.rs` — `TenantSubscriber` actor
- `crates/frf-librefang/src/error.rs` — `LibreFangError`
- `Cargo.toml` (workspace) — add `frf-librefang` member + bossfang dep
- `crates/frf-librefang/tests/bus_smoke.rs` — integration: publish → subscribe delivers event

## Acceptance Criteria

- [ ] `cargo check -p frf-librefang` clean
- [ ] Supervisor starts without panicking (test)
- [ ] publish → subscribe smoke test: event arrives at subscriber within 100ms
- [ ] `AgentEventBus` trait is fully implemented (all methods)
- [ ] No `unwrap()` — all errors returned via `PortError`
- [ ] Supervisor restarts publisher actor on failure (test)
- [ ] `clippy::pedantic` passes
