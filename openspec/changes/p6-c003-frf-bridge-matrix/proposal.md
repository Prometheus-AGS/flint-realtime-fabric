# p6-c003 — frf-bridge-matrix: Tuwunel Matrix Homeserver Bridge

## Summary

Create `crates/frf-bridge-matrix/` implementing the `FederationBridge` port
trait for `FederationProtocol::Matrix`. The adapter subscribes to Matrix room
events via a Tuwunel homeserver client and projects them onto the FRF spine as
`FederatedEvent` frames.

## Open Decision Resolution

**Tuwunel dependency**: No published crate exists on crates.io as of the plan
date. The GQAdonis fork at `https://github.com/GQAdonis/tuwunel` is the target
but its Cargo package interface is unconfirmed. Resolution: stub the homeserver
client behind a `MatrixClient` trait in `frf-bridge-matrix`, with a concrete
`TuwunelClient` impl using the git dep when available. If the git dep fails
resolution during build, a `MockMatrixClient` (feature-gated `cfg(test)`) is
the fallback used by integration tests.

Concrete Cargo.toml strategy:
```toml
[dependencies]
# Try git dep; if unresolvable, use reqwest-based REST stub
# tuwunel = { git = "https://github.com/GQAdonis/tuwunel", optional = true }
reqwest = { workspace = true, features = ["json", "stream"] }
```

The `MatrixClient` trait abstracts over both so the bridge compiles and tests
pass regardless of Tuwunel availability.

## Design

### Port implementation

```rust
// crates/frf-bridge-matrix/src/lib.rs
pub struct MatrixBridge {
    client: Arc<dyn MatrixClient + Send + Sync>,
    room_id: String,
}

#[async_trait]
impl FederationBridge for MatrixBridge {
    async fn send(&self, event: FederatedEvent) -> Result<(), FederationError> { ... }

    fn subscribe(&self) -> FederationStream {
        Box::pin(self.client.room_event_stream(self.room_id.clone()))
    }
}
```

### `MatrixClient` trait (internal)

```rust
#[async_trait]
pub trait MatrixClient: Send + Sync {
    fn room_event_stream(&self, room_id: String) -> impl Stream<Item = Result<FederatedEvent, FederationError>> + Send;
    async fn send_event(&self, room_id: &str, content: serde_json::Value) -> Result<(), FederationError>;
}
```

### Event projection

Matrix room event → `FederatedEvent`:
- `protocol: FederationProtocol::Matrix`
- `source: format!("matrix:{room_id}/{event_id}")`
- `envelope: EventEnvelope { ... }` — maps Matrix event `content` to
  `EventEnvelope.payload`; `sender` maps to `entity_id`

### Workspace wiring

Add `frf-bridge-matrix` to root `Cargo.toml` `[workspace.members]`. The crate
is NOT added to `frf-gateway`'s `[dependencies]` in this change — gateway
wiring happens in p6-c006.

## Files Affected

- `Cargo.toml` (workspace) — add member
- `crates/frf-bridge-matrix/Cargo.toml` (NEW)
- `crates/frf-bridge-matrix/src/lib.rs` (NEW)
- `crates/frf-bridge-matrix/src/client.rs` (NEW — `MatrixClient` trait + `ReqwestMatrixClient`)
- `crates/frf-bridge-matrix/src/convert.rs` (NEW — Matrix event → `FederatedEvent`)
- `crates/frf-bridge-matrix/src/error.rs` (NEW — `MatrixBridgeError` via `thiserror`)

## Quality Gates

- [ ] `cargo check --workspace` passes (including new crate)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes
- [ ] `cargo test -p frf-bridge-matrix` passes (unit tests on projection logic)
- [ ] No `unwrap()` in library crate
- [ ] `MatrixBridgeError` uses `thiserror` and implements `FederationError` conversion
- [ ] `FederationBridge` impl is the ONLY port this crate implements
