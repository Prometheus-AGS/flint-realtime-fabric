# p3-c002 — `OpStore` port trait in `frf-ports`

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c001

## Directory
`crates/frf-ports/src/`

## What this change does

Defines the `OpStore` port trait — the seam between the on-device op-log adapter
(`frf-store-redb`) and the application layer. This is a `frf-ports` addition with
semver impact.

### Trait interface

```rust
/// An op that has not yet been confirmed by the server.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PendingOp {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub payload: Bytes,          // opaque CRDT binary (engine-agnostic)
    pub local_seq: u64,          // monotonic, device-local
}

#[async_trait]
pub trait OpStore: Send + Sync + 'static {
    /// Durably append an outgoing op before it is sent.
    async fn queue_op(&self, op: PendingOp) -> Result<(), PortError>;
    /// Return all queued ops for an entity in local_seq order.
    async fn drain_pending(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
    ) -> Result<Vec<PendingOp>, PortError>;
    /// Mark ops with local_seq <= confirmed_seq as synced (safe to drop).
    async fn mark_synced(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
        confirmed_seq: u64,
    ) -> Result<(), PortError>;
}
```

### Semver note

`frf-ports` is pre-1.0 so this is a minor addition. Adding a new public trait
does not break existing trait implementors.

## Non-goals

- Does not implement the trait (that is p3-c004).
- Does not touch `frf-domain` or any adapter crate.
- Does not add `async_trait` if `frf-ports` already has it — reuse existing dep.
