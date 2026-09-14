# p6-c004 — frf-bridge-atproto: ATProto Jetstream Bridge

## Summary

Create `crates/frf-bridge-atproto/` implementing the `FederationBridge` port
trait for `FederationProtocol::AtProto`. The adapter consumes ATProto Jetstream
SSE events and projects them onto the FRF spine as `FederatedEvent` frames.

## Open Decision Resolution

**Tranquil / Jetstream Rust client**: No `tranquil` crate published on crates.io
as of plan date. No known tokio-native Jetstream consumer crate exists. Resolution:
implement SSE consumer directly against the Bluesky public Jetstream API
(`wss://jetstream2.us-east.bsky.network/subscribe`) using:
- `reqwest` (HTTP) or `tokio-tungstenite` (WebSocket) for transport
- `eventsource-stream = "0.2"` (SSE) or custom WS message framing
- `serde_json` for CBOR/JSON event deserialization

Bluesky Jetstream uses WebSocket, not SSE. Use `tokio-tungstenite` with
`tokio-tungstenite = { workspace = true }` or add to workspace deps.

## Design

### Port implementation

```rust
// crates/frf-bridge-atproto/src/lib.rs
pub struct AtProtoBridge {
    jetstream_url: String,
    collections: Vec<String>, // filter by lexicon type e.g. "app.bsky.feed.post"
}

#[async_trait]
impl FederationBridge for AtProtoBridge {
    async fn send(&self, _event: FederatedEvent) -> Result<(), FederationError> {
        // ATProto write path deferred — Jetstream is read-only subscription
        Err(FederationError::Unsupported("ATProto write not implemented".into()))
    }

    fn subscribe(&self) -> FederationStream {
        Box::pin(jetstream_stream(self.jetstream_url.clone(), self.collections.clone()))
    }
}
```

### Jetstream stream

```rust
async fn jetstream_stream(url: String, collections: Vec<String>)
    -> impl Stream<Item = Result<FederatedEvent, FederationError>>
```

Connects via `tokio-tungstenite`, sends a subscription JSON frame:
```json
{ "wantedCollections": ["app.bsky.feed.post"] }
```
Maps each WebSocket message to `FederatedEvent`:
- `protocol: FederationProtocol::AtProto`
- `source: format!("atproto:{did}/{rkey}")`
- `envelope` — maps Jetstream `commit.record` → `EventEnvelope.payload`

### Event projection

ATProto Jetstream commit event → `FederatedEvent`:
- `protocol: FederationProtocol::AtProto`
- `source: "{event.did}/{event.commit.rkey}"`
- `envelope.entity_id = event.did`
- `envelope.payload = serde_json::to_value(event.commit.record)`

## Files Affected

- `Cargo.toml` (workspace) — add member + `tokio-tungstenite` if not present
- `crates/frf-bridge-atproto/Cargo.toml` (NEW)
- `crates/frf-bridge-atproto/src/lib.rs` (NEW)
- `crates/frf-bridge-atproto/src/jetstream.rs` (NEW — WS consumer)
- `crates/frf-bridge-atproto/src/convert.rs` (NEW — ATProto event → `FederatedEvent`)
- `crates/frf-bridge-atproto/src/error.rs` (NEW — `AtProtoBridgeError` via `thiserror`)

## Quality Gates

- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes
- [ ] `cargo test -p frf-bridge-atproto` passes (unit tests on projection; WS test gated by `#[cfg(feature = "integration")]`)
- [ ] No `unwrap()` in library crate
- [ ] Write path returns `Err(FederationError::Unsupported(...))` — not panics
- [ ] `FederationBridge` impl is the ONLY port this crate implements
