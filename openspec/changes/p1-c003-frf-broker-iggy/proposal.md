# p1-c003 — frf-broker-iggy: LogBroker → Apache Iggy

## Affected crates
- `crates/frf-broker-iggy` (new — stub created by p1-c001)

## Dependency-rule impact
Layer 2 (infrastructure adapter). Imports `frf-domain` and `frf-ports`. Implements `LogBroker`. Must NOT import `frf-app` or any other adapter crate.

## What this change does

Implements the `LogBroker` port against the GQAdonis Apache Iggy fork.

### Channel mapping
```
LogBroker Channel  →  Iggy
channel.tenant_id  →  stream name  (one stream per tenant)
channel.path       →  topic name   (e.g. "entity/user/updates")
consumer_id hash   →  partition_id (consistent hash for rebalancing)
```

### `IggyBroker` struct
```rust
pub struct IggyBroker {
    client: Arc<IggyClient>,
}
impl IggyBroker {
    pub async fn new(connection_string: &str) -> anyhow::Result<Self>
}
```

### `LogBroker` implementation
- `publish`: serialize `EventEnvelope` → JSON bytes → `producer.send()`
- `subscribe`: spawn Iggy consumer → wrap in `async_stream::stream!` poll loop → return `EventStream`
- `seek`: call Iggy `store_consumer_offset`
- `ack`: call Iggy `store_consumer_offset`
- `ensure_channel`: `create_stream` + `create_topic` idempotently (ignore AlreadyExists errors)

### Backpressure design
The poll loop uses a bounded `tokio::sync::mpsc::channel(256)`. If the receiver drops (client disconnects), the sender task exits cleanly. The `EventStream` is a `ReceiverStream` wrapper.

### Module layout
```
crates/frf-broker-iggy/src/
├── lib.rs
├── broker.rs     IggyBroker + LogBroker impl
├── channel.rs    Channel → Iggy name mapping helpers
└── error.rs      IggyError → PortError conversion
```

## Phase 1 exit criterion satisfied
`frf-broker-iggy` unit tests pass; integration test publishes and subscribes against a real local Iggy instance (or test container).

## Non-goals
- Does not implement `CrdtStore`, `MediaSignaler`, or any other port.
- Does not run the Iggy server itself — requires an external Iggy instance.
- Does not implement clustering (single-node Iggy; clustering planned for Iggy roadmap).
