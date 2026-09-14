# p5-c001 — AgentEventBus port trait

## Summary

Add `AgentEventBus` port trait to `frf-ports`. This is the seam that allows
`frf-librefang` (BossFang) to be wired in without `frf-app` or `frf-gateway`
knowing about ractor directly.

## Motivation

`frf-ports` is the Layer 1 seam for all infrastructure adapters. Before any
BossFang-specific code can exist, the abstract port contract must be defined here.
Everything downstream (agentproto, librefang, gateway) depends on this contract.

## Design

```rust
// frf-ports/src/agent_bus.rs

use async_trait::async_trait;
use frf_domain::AgentEvent;

#[async_trait]
pub trait AgentEventBus: Send + Sync + 'static {
    /// Publish an agent event to the bus.
    async fn publish(&self, event: AgentEvent) -> Result<(), PortError>;

    /// Subscribe to agent events filtered by tenant.
    /// Returns an async stream of events.
    async fn subscribe(
        &self,
        tenant_id: &str,
    ) -> Result<AgentEventStream, PortError>;
}

pub type AgentEventStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = AgentEvent> + Send>>;
```

Re-export from `frf-ports/src/lib.rs`:
```rust
pub mod agent_bus;
pub use agent_bus::{AgentEventBus, AgentEventStream};
```

## Files Changed

- `crates/frf-ports/src/agent_bus.rs` — NEW trait file
- `crates/frf-ports/src/lib.rs` — add `pub mod agent_bus` + re-exports
- `crates/frf-ports/Cargo.toml` — add `async-trait`, `futures` deps

## Acceptance Criteria

- [ ] `cargo check -p frf-ports` passes
- [ ] `AgentEventBus` is exported from `frf_ports` crate root
- [ ] Trait is object-safe (no generics on methods)
- [ ] `#[non_exhaustive]` NOT needed here (trait, not enum)
- [ ] No `unwrap()` in library code
- [ ] `clippy::pedantic` passes
