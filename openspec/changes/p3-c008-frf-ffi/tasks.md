# Tasks — p3-c008 frf-ffi

- [ ] **T1** Create `crates/frf-ffi/Cargo.toml`
  - `[lib] crate-type = ["cdylib", "staticlib"]` (required for UniFFI)
  - Dependencies: `frf-crdt = { path = "../frf-crdt" }`, `frf-ports = { path = "../frf-ports" }`, `uniffi = { workspace = true }`, `thiserror = { workspace = true }`, `tokio = { workspace = true, features = ["rt-multi-thread"] }`, `bytes = { workspace = true }`, `tracing = { workspace = true }`
  - `[build-dependencies] uniffi = { workspace = true, features = ["build"] }`
  - `[lints.clippy] pedantic = "warn"`
  - Verification: `cargo check -p frf-ffi` exits 0

- [ ] **T2** Create `crates/frf-ffi/build.rs`
  - Call `uniffi::generate_scaffolding("src/frf.udl")` — OR use proc-macro only
  - If proc-macro only: empty `build.rs` is fine; scaffolding is via `uniffi::setup_scaffolding!("frf")` in `lib.rs`
  - Verification: `cargo build -p frf-ffi` exits 0

- [ ] **T3** Create `crates/frf-ffi/src/error.rs`
  - Define `FrfError` enum with `#[derive(uniffi::Error, Debug, thiserror::Error)]` and `#[uniffi(flat_error)]`
  - Variants: `Connect { message: String }`, `Crdt { message: String }`, `Publish { message: String }`
  - Verification: compiles with no unused variant warnings

- [ ] **T4** Create `crates/frf-ffi/src/callback.rs`
  - Define `EventCallback` trait with `#[uniffi::export(callback_interface)]`
  - Methods: `on_event(payload: Vec<u8>, offset: u64)`, `on_error(message: String)`, `on_disconnected()`
  - Verification: compiles

- [ ] **T5** Create `crates/frf-ffi/src/client.rs`
  - Define `FrfClient` struct (wraps `Arc<InnerClient>` where `InnerClient` holds a `tokio::Runtime`)
  - `#[uniffi::export]` on the `impl FrfClient` block
  - `new(gateway_url: String) -> Result<FrfClient, FrfError>` — create a dedicated Tokio runtime; store `Arc<Runtime>`
  - `subscribe(channel_id, consumer_id, callback: Box<dyn EventCallback>) -> Result<SubscriptionHandle, FrfError>` — spawn task on runtime; callback is called on each event
  - `publish(channel_id, payload: Vec<u8>) -> Result<(), FrfError>` — block_on publish
  - `sync_apply(entity_id, tenant_id, delta: Vec<u8>) -> Result<Vec<u8>, FrfError>` — delegates to `frf_crdt::apply_delta`
  - No `unwrap()` / `expect()`
  - Verification: `cargo check -p frf-ffi` exits 0

- [ ] **T6** Create `crates/frf-ffi/src/lib.rs`
  - `uniffi::setup_scaffolding!("frf");`
  - `pub mod error; pub mod callback; pub mod client;`
  - `pub use error::FrfError; pub use callback::EventCallback; pub use client::{FrfClient, SubscriptionHandle};`
  - `#[uniffi::export] pub fn frf_version() -> String { env!("CARGO_PKG_VERSION").to_string() }`
  - Verification: `cargo build -p frf-ffi` exits 0; produces `libfrf_ffi.{dylib,a}`

- [ ] **T7** Run UniFFI bindgen to confirm binding generation works
  - `cargo run --bin uniffi-bindgen generate --library target/debug/libfrf_ffi.dylib --language swift --out-dir /tmp/frf-ffi-swift-check`
  - Verify: `/tmp/frf-ffi-swift-check/frf.swift` exists
  - `cargo run --bin uniffi-bindgen generate --library target/debug/libfrf_ffi.dylib --language kotlin --out-dir /tmp/frf-ffi-kotlin-check`
  - Verify: `/tmp/frf-ffi-kotlin-check/uniffi/frf/frf.kt` exists
  - Verification: both files exist and are non-empty

- [ ] **T8** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
