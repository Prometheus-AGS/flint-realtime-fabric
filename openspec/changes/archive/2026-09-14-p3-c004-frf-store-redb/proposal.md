# p3-c004 — `frf-store-redb` crate — on-device op-log

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c002 (OpStore port trait), p3-c001 (redb dep pinned)

## Directory
`crates/frf-store-redb/`

## What this change does

Creates the `frf-store-redb` crate — a single-port adapter implementing `OpStore`
using redb 4.1.0. Provides durable, append-only storage for unsynced `PendingOp`
entries on-device. Used by mobile clients and desktop apps.

### Adapter: `RedbOpStore`

```rust
pub struct RedbOpStore {
    db: Arc<redb::Database>,
}
```

Implements `OpStore` from `frf-ports`:
- `queue_op` — insert `PendingOp` into a redb table keyed by `(tenant_id, entity_id, local_seq)`
- `drain_pending` — range-scan and return all pending ops for an entity in order
- `mark_synced` — delete all rows with `local_seq <= confirmed_seq`

### Table schema

```
TABLE: "pending_ops"
KEY:   (tenant_id: &str, entity_id: &str, local_seq: u64)  → fixed-width u64 for ordering
VALUE: payload: &[u8]   (opaque Loro binary delta)
```

One table, single-writer, no transactions spanning reconnect — each op is
independently durable.

### Error handling

Map `redb::Error` to `PortError::Internal` via a `From` impl. No `unwrap()` or
`expect()` in library code.

## Non-goals

- Does not implement browser persistence (IndexedDB via `frf-wasm` — Phase 4).
- Does not implement `CrdtStore` (server-side checkpoints live in `frf-store-surreal`).
- Does not run on SurrealDB or any networked store.
