# p3-c008 — `frf-ffi` crate — UniFFI scaffold

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c007 (full sync stack wired in gateway)

## Directory
`crates/frf-ffi/`

## What this change does

Creates `frf-ffi` — the UniFFI 0.31.2 scaffold that exports a thin, FFI-safe
Rust API surface to Swift, Kotlin, and Dart. Business logic, CRDT merge, and
reconnection stay Rust-side. Only stable, FFI-safe types cross the boundary.

### Approach

UniFFI proc-macro only (`#[uniffi::export]`) — no `.udl` file. This reduces
maintenance surface as the API evolves.

### Exported API surface (initial)

```rust
#[uniffi::export]
pub fn frf_version() -> String;

#[uniffi::export]
pub fn apply_crdt_delta(existing: Vec<u8>, delta: Vec<u8>) -> Result<Vec<u8>, FrfError>;

#[uniffi::export]
pub struct FrfClient { /* internal Arc<...> */ }

#[uniffi::export]
impl FrfClient {
    pub fn new(gateway_url: String) -> Result<FrfClient, FrfError>;

    /// Subscribe to an entity channel. Calls callback on each event.
    pub fn subscribe(
        &self,
        channel_id: String,
        consumer_id: String,
        callback: Box<dyn EventCallback>,
    ) -> Result<SubscriptionHandle, FrfError>;

    pub fn publish(
        &self,
        channel_id: String,
        payload: Vec<u8>,
    ) -> Result<(), FrfError>;

    pub fn sync_apply(
        &self,
        entity_id: String,
        tenant_id: String,
        delta: Vec<u8>,
    ) -> Result<Vec<u8>, FrfError>;
}

#[uniffi::export(callback_interface)]
pub trait EventCallback: Send + Sync {
    fn on_event(&self, payload: Vec<u8>, offset: u64);
    fn on_error(&self, message: String);
    fn on_disconnected(&self);
}
```

### Error type

```rust
#[derive(uniffi::Error, Debug, thiserror::Error)]
#[uniffi(flat_error)]
pub enum FrfError {
    #[error("connect: {message}")]
    Connect { message: String },
    #[error("crdt: {message}")]
    Crdt { message: String },
    #[error("publish: {message}")]
    Publish { message: String },
}
```

### Build script (`build.rs`)

```rust
fn main() {
    uniffi::generate_scaffolding("src/frf.udl").unwrap_or(());
    // proc-macro approach: no udl needed — call uniffi::setup_scaffolding!()
}
```

Use `uniffi::setup_scaffolding!("frf")` macro in `lib.rs`.

## Non-goals

- Does not generate Swift/Kotlin/Dart code (that happens in p3-c009/010/011).
- Does not expose the full `frf-domain` type graph — only the minimal subset
  safe for FFI consumers.
- Does not expose admin-UI or gateway configuration surface.
