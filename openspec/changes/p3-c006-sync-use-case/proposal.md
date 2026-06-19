# p3-c006 — `SyncUseCase` in `frf-app`

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c003 (frf-crdt: apply_delta, LoroCrdtStore), p3-c004 (frf-store-redb: RedbOpStore)

## Directory
`crates/frf-app/src/`

## What this change does

Adds `SyncUseCase` to `frf-app` — the engine-agnostic application-layer handler
for incoming CRDT sync operations. It wires `CrdtStore` (checkpoint/restore) and
`OpStore` (op-log drain/mark_synced) via port injection.

### Dependency rule check

`frf-app` imports only `frf-ports` traits and `frf-domain` types. It must NOT
import `frf-crdt` or `frf-store-redb` directly — those are injected at wiring
time in `frf-gateway`.

### Interface

```rust
pub struct SyncUseCase<C: CrdtStore, O: OpStore> {
    crdt_store: Arc<C>,
    op_store: Arc<O>,
}

impl<C, O> SyncUseCase<C, O>
where
    C: CrdtStore,
    O: OpStore,
{
    /// Apply an incoming delta to the stored CRDT state.
    /// Returns the updated encoded snapshot to send back to the client.
    pub async fn apply_incoming(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
        delta: &[u8],
    ) -> Result<Vec<u8>, AppError>;

    /// Flush pending ops for an entity (called on reconnect).
    /// Returns ops to be sent to the server in order.
    pub async fn flush_pending(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
    ) -> Result<Vec<PendingOp>, AppError>;

    /// Confirm server acknowledgment up to confirmed_seq.
    pub async fn confirm_synced(
        &self,
        entity_id: &EntityId,
        tenant_id: &TenantId,
        confirmed_seq: u64,
    ) -> Result<(), AppError>;
}
```

Note: `apply_incoming` calls `frf-crdt::apply_delta` — but via a **function pointer
or trait object** injected at construction, not via a direct `frf-crdt` import.
Use `Box<dyn Fn(&[u8], &[u8]) -> Result<Vec<u8>, _> + Send + Sync>` or a thin
`ApplyDelta` trait to maintain the dependency inversion.

## Non-goals

- Does not expose gRPC (that is `frf-gateway`, p3-c007).
- Does not know about Loro — only `&[u8]` deltas.
- Does not manage WebSocket sessions.
