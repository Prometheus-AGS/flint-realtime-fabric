//! FFI surface for Flint Realtime Fabric (Swift, Kotlin, Dart via `UniFFI`).
//!
//! Two layers:
//! - **CRDT** (`crdt`): thin sync functions over `frf-crdt`, operating on opaque
//!   `Vec<u8>` byte blobs so the engine encoding is never part of the contract.
//! - **Transport** (`client`): [`FrfFfiClient`] wraps the hand-written Rust SDK
//!   (`frf-sdk-rust`) for real connect / auth / publish / subscribe. Async methods
//!   run on the tokio runtime; domain types cross as JSON strings.
//!
//! Build with `crate-type = ["cdylib", "staticlib"]` — see `Cargo.toml`.

uniffi::setup_scaffolding!("frf");

pub mod client;
pub mod crdt;
pub mod error;

pub use client::{EventCallback, FrfFfiClient};
pub use crdt::{crdt_apply_delta, crdt_new_snapshot, crdt_snapshot_version};
pub use error::{ClientFfiError, CrdtFfiError};
