# p7-c001 — `frf-media-str0m` Sovereign SFU Crate

## Summary

Create `crates/frf-media-str0m/` implementing the `MediaSignaler` port trait
using the `str0m 0.7` sans-I/O WebRTC crate. This is the sovereign SFU
adapter — no cloud dependency, all ICE/DTLS negotiation runs in-process.

## Motivation

Phase 4 implemented `frf-media-livekit` (hosted SFU). CLAUDE.md requires a
sovereign SFU option (`str0m`). The `MediaSignaler` port is already defined;
this change adds the str0m implementation behind it.

## Changes

### `Cargo.toml` (workspace)
- Add `str0m = "0.7"` to `[workspace.dependencies]`

### `crates/frf-media-str0m/Cargo.toml` (NEW)
```toml
[package]
name = "frf-media-str0m"
version.workspace = true
edition.workspace = true

[lints.clippy]
pedantic = "warn"

[dependencies]
frf-domain = { path = "../frf-domain" }
frf-ports = { path = "../frf-ports" }
str0m = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread", "sync"] }
tokio-stream = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
bytes = { workspace = true }
```

### `crates/frf-media-str0m/src/lib.rs` (NEW)
```rust
#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod sfu;

pub use error::StrOmError;
pub use sfu::StrOmSignaler;
```

### `crates/frf-media-str0m/src/error.rs` (NEW)
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StrOmError {
    #[error("ICE negotiation failed: {0}")]
    Ice(String),
    #[error("DTLS handshake failed: {0}")]
    Dtls(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("port error: {0}")]
    Port(#[from] frf_ports::PortError),
}
```

### `crates/frf-media-str0m/src/sfu.rs` (NEW)

Implement `MediaSignaler` for `StrOmSignaler`:
- `start_session(session_id)` — creates a new `str0m::Rtc` instance keyed by session_id
- `send_signal(session_id, signal_envelope)` — deserializes SDP offer/answer or ICE candidate from `SignalEnvelope.payload`, feeds into `str0m::Rtc`
- `receive_signal(session_id)` — returns stream of `SignalEnvelope` from `str0m::Rtc` output events (ICE candidates, SDP answer)
- Sessions stored in `DashMap<SessionId, Arc<Mutex<str0m::Rtc>>>`

### `Cargo.toml` (workspace members)
- Add `"crates/frf-media-str0m"` to `[workspace.members]`

## Quality Gates

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
- `cargo test -p frf-media-str0m`
