# p3-c003 — `frf-crdt` crate — Loro adapter implementing `CrdtStore`

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c002 (OpStore port, frf-ports updated), p3-c001 (Loro dep pinned)

## Directory
`crates/frf-crdt/`

## What this change does

Creates the `frf-crdt` crate — a single-port adapter implementing `CrdtStore`
using the Loro 1.13.1 CRDT engine. Encodes and decodes Loro binary snapshots as
the `encoded: Bytes` field on `CrdtSnapshot`. Provides a merge function that
applies an incoming `SyncOp.payload` (Loro binary delta) to an existing Loro doc.

### Adapter: `LoroCrdtStore`

```rust
pub struct LoroCrdtStore<S> { inner: Arc<S> }
// S: inner kv-store for the binary blobs (trait-injected for testability)
```

`LoroCrdtStore` implements `CrdtStore` from `frf-ports`:
- `checkpoint` — serialize the current Loro doc state to `encoded: Bytes`; store via `inner`
- `restore` — load `encoded` bytes; deserialize into a Loro doc; return `CrdtSnapshot`
- `purge` — delete the stored blob via `inner`

### Engine-agnostic merge surface

Expose a free function:

```rust
/// Apply a Loro binary delta to an existing Loro doc.
/// Returns the merged doc serialized to bytes.
pub fn apply_delta(existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, CrdtError>;
```

This is the function `frf-ffi` calls when forwarding a `SyncOp` to the device's
local doc.

### Error type

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CrdtError {
    #[error("loro decode: {0}")]
    Decode(String),
    #[error("loro merge: {0}")]
    Merge(String),
    #[error("store: {0}")]
    Store(#[from] PortError),
}
```

## Non-goals

- Does not implement `OpStore` (that is `frf-store-redb`, p3-c004).
- Does not implement the server-side persistence (that is `frf-store-surreal`, p3-c005).
- Does not expose FFI surface (that is `frf-ffi`, p3-c008).
- Does not touch `frf-domain`, `frf-app`, or `frf-gateway`.
