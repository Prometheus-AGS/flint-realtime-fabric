# p1-c006 — frf-postgres-cdc: WAL Logical Replication → Spine

## Affected crates
- `crates/frf-postgres-cdc` (new — stub created by p1-c001)

## Dependency-rule impact
Layer 2 (infrastructure adapter). Imports `frf-domain` and `frf-ports`. Implements the CDC-to-spine data path. Must NOT import `frf-app`, `frf-broker-iggy`, or any other adapter crate — it receives a `Box<dyn LogBroker>` at construction time (dependency injection).

## What this change does

Implements a Postgres logical replication consumer that decodes `pgoutput` WAL events and publishes them to the durable event spine via `LogBroker`.

### Data flow
```
Postgres WAL (logical replication slot, pgoutput plugin)
  │
  ├─ INSERT → EntityChange { op: Insert, entity_type, data, ... }
  ├─ UPDATE → EntityChange { op: Update, data, previous, ... }
  └─ DELETE → EntityChange { op: Delete, previous, ... }
  │
  ▼ serialize to EventEnvelope { kind: EntityChange, payload: serde_json::Value }
  │
  ▼ LogBroker.publish(envelope) → Offset
  │
  ▼ store_lsn(confirmed_lsn) → heartbeat to Postgres (keeps slot active)
```

### `CdcConfig`
```rust
pub struct CdcConfig {
    pub database_url: String,
    pub slot_name: String,
    pub publication_name: String,
    pub tenant_id: TenantId,
    pub channel_path: String,     // e.g. "entity/changes"
}
```

### `PostgresCdcConsumer` struct
```rust
pub struct PostgresCdcConsumer<L: LogBroker> {
    config: CdcConfig,
    broker: Arc<L>,
}
impl<L: LogBroker> PostgresCdcConsumer<L> {
    pub fn new(config: CdcConfig, broker: Arc<L>) -> Self
    pub async fn run_until_shutdown(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) -> anyhow::Result<()>
}
```

### WAL decode approach
Use `tokio-postgres` with the `pgoutput` logical decoding plugin. The `tokio-postgres::SimpleQueryStream` can be driven with `START_REPLICATION SLOT ... LOGICAL ...` commands. This avoids the `pg_replicate` crate dependency until its maintenance status is verified.

### LSN checkpointing
Every 1000 messages (configurable) or on clean shutdown: send `StandbyStatusUpdate` to Postgres to advance the replication slot LSN. This prevents WAL bloat.

### Module layout
```
crates/frf-postgres-cdc/src/
├── lib.rs
├── consumer.rs    PostgresCdcConsumer + run_until_shutdown
├── decode.rs      pgoutput WAL message → EntityChange decoder
└── config.rs      CdcConfig
```

## Phase 1 exit criterion satisfied
`decode.rs` unit tests decode known WAL byte sequences to `EntityChange`. Integration test (marked `#[ignore]`) runs against a real Postgres with a test replication slot.

## Non-goals
- Does not implement multi-table fan-out filtering (all tables in the publication are captured).
- Does not handle DDL changes (schema migrations) — consumer panics on unexpected message types, which is correct behavior (restart required).
- Does not implement the SurrealDB checkpoint store (`frf-store-surreal` is Phase 3).
