//! FFI scaffold for Flint Realtime Fabric.
//!
//! Exposes a thin, sync surface over `frf-crdt` for `UniFFI` code generation
//! (Swift, Kotlin) and `flutter_rust_bridge` (Dart). All functions operate on
//! opaque `Vec<u8>` CRDT byte blobs so the engine encoding is never part of
//! the public FFI contract.
//!
//! Build with `crate-type = ["cdylib", "staticlib"]` — see `Cargo.toml`.

uniffi::setup_scaffolding!("frf");

pub mod crdt;
pub mod error;

pub use crdt::{crdt_apply_delta, crdt_new_snapshot, crdt_snapshot_version};
pub use error::CrdtFfiError;
