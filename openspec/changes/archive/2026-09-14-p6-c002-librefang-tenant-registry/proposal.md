# p6-c002 — LibreFangBus TenantActorRegistry

## Summary

Replace the single global `PublisherActor` in `frf-librefang` with a
`TenantActorRegistry` that maintains one `PublisherActor` per active tenant,
with lazy initialization and idle-timeout eviction.

## Motivation

Phase 5 introduced a single `PublisherActor` mailbox that handles all tenant
subscriptions. Under concurrent load, all tenant events queue on one actor,
creating a throughput bottleneck proportional to the number of tenants. The
`TenantActorRegistry` shards the actor tree by tenant, isolating mailbox
queues and enabling per-tenant capacity limits.

## Design

### `TenantActorRegistry` (`frf-librefang/src/registry.rs`)

```rust
use dashmap::DashMap;
use ractor::{Actor, ActorRef};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

const IDLE_EVICTION_SECS: u64 = 300; // 5 minutes

struct TenantEntry {
    actor: ActorRef<PublisherMessage>,
    last_active: Instant,
}

pub struct TenantActorRegistry {
    entries: Arc<DashMap<String, TenantEntry>>,
}
```

Key behaviors:
- `get_or_create(tenant_id: &str) -> ActorRef<PublisherMessage>` — lazy init
- Background eviction task sweeps idle entries every 60s
- Actor spawn uses `Actor::spawn_linked` under the root supervisor
- `DashMap` provides lock-free concurrent access per shard

### `LibreFangBus` update (`frf-librefang/src/bus.rs`)

Replace `publisher: ActorRef<PublisherMessage>` with
`registry: Arc<TenantActorRegistry>`.

`publish(&self, tenant_id, event)` → `registry.get_or_create(tenant_id).send_message(...)`.

`subscribe(&self, tenant_id)` → actor for tenant receives `Subscribe` message.

### Workspace dependency

Add `dashmap = "6"` to `[workspace.dependencies]` in root `Cargo.toml` and
to `frf-librefang/Cargo.toml`.

## Files Affected

- `Cargo.toml` (workspace) — add `dashmap`
- `crates/frf-librefang/Cargo.toml` — add `dashmap`
- `crates/frf-librefang/src/registry.rs` (NEW)
- `crates/frf-librefang/src/bus.rs` (MODIFY)
- `crates/frf-librefang/src/lib.rs` (MODIFY — re-export `TenantActorRegistry`)

## Quality Gates

- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes
- [ ] `cargo test -p frf-librefang` passes (existing tests must continue to pass)
- [ ] No `unwrap()` in `registry.rs`
- [ ] `TenantActorRegistry` is `#[must_use]` on construction
