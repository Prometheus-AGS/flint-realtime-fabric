# p3-c001 — Loro CRDT engine ADR + workspace dependency pins

## Phase
phase-3-ffi-sdks-crdt

## Depends on
—

## Directory
`docs/decisions/`, `Cargo.toml` (workspace)

## What this change does

Commits the CRDT engine choice as an Architecture Decision Record and pins all
Phase 3 Cargo dependencies in the workspace manifest. This is the blocker gate
for every other Phase 3 change.

### Decision committed

**Loro 1.13.1** is selected over automerge 0.10.0.

Rationale:
- `loro-ffi 1.13.1` provides first-party UniFFI bindings for Swift + Kotlin —
  removing the need to hand-roll `#[uniffi::export]` wrappers around automerge.
- Loro's binary Sync Protocol is incremental and mergeable with the same
  properties as Automerge Sync Protocol.
- Loro is 1.x stable; automerge is pre-1.0.
- Automerge has no `loro-ffi` equivalent — it would require manual FFI shims,
  equivalent work with less upstream support.

### Cargo.toml additions

```toml
# Phase 3 dependencies pinned at workspace level
loro       = { version = "1.13.1" }
loro-ffi   = { version = "1.13.1" }
redb       = { version = "4.1.0" }
surrealdb  = { version = "3.1.5" }
uniffi     = { version = "0.31.2", features = ["build"] }
uniffi_macros = { version = "0.31.2" }
```

New workspace members to add:
```toml
"crates/frf-crdt",
"crates/frf-store-redb",
"crates/frf-store-surreal",
"crates/frf-ffi",
```

## Non-goals

- Does not implement any CRDT logic (that is p3-c003).
- Does not add `frf-wasm` (deferred to Phase 4).
- Does not change any existing crate.
