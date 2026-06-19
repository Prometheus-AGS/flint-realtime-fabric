# p3-c005 — `frf-store-surreal` crate — server-side CRDT checkpoint store

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c001 (surrealdb dep pinned, workspace membership)

## Directory
`crates/frf-store-surreal/`

## What this change does

Creates the `frf-store-surreal` crate — a single-port adapter implementing
`CrdtStore` using SurrealDB 3.1.5. Stores CRDT snapshots server-side, keyed by
`(entity_id, tenant_id)`. The gateway uses this to checkpoint merged state so a
cold-starting device can restore from a known version.

### Adapter: `SurrealCrdtStore`

```rust
pub struct SurrealCrdtStore {
    db: Arc<surrealdb::Surreal<surrealdb::engine::remote::ws::Client>>,
    namespace: String,
    database: String,
}
```

Implements `CrdtStore` from `frf-ports`:
- `checkpoint` — upsert a SurrealDB record `crdt_snapshots:<entity_id>` with `encoded`, `version`, `tenant_id`, `updated_at`
- `restore` — SELECT record; decode `encoded` field; return `CrdtSnapshot`
- `purge` — DELETE record

### Schema (SurrealDB table)

```sql
DEFINE TABLE crdt_snapshots SCHEMALESS;
DEFINE INDEX crdt_tenant_entity ON crdt_snapshots FIELDS tenant_id, entity_id;
```

Table creation is idempotent — called at `SurrealCrdtStore::connect()`.

## Non-goals

- Does not implement `OpStore` (that is `frf-store-redb`).
- Does not run SurrealDB (integration test uses a Docker container or feature-gate).
- Does not provide server query APIs beyond `CrdtStore`.
- Integration tests are feature-gated: `#[cfg(feature = "integration")]` so unit CI always passes.
