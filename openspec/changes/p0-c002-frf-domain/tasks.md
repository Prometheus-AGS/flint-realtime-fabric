# Tasks — p0-c002 frf-domain

**Semver note:** All public types in this crate are load-bearing for frf-ports, frf-proto, frf-app, and all SDKs. Treat every signature change as semver minor (additive) or major (breaking).

- [ ] **T1** Create `crates/frf-domain/Cargo.toml`
  - File: `crates/frf-domain/Cargo.toml`
  - `[dependencies]`: `serde = { workspace = true, features = ["derive"] }`, `serde_json = { workspace = true }`, `uuid = { workspace = true, features = ["v4", "serde"] }`, `chrono = { workspace = true, features = ["serde"] }`
  - Add `crates/frf-domain` to `[workspace.members]` in root `Cargo.toml`
  - Verification: `cargo check -p frf-domain` exits 0

- [ ] **T2** Create `crates/frf-domain/src/ids.rs`
  - File: `crates/frf-domain/src/ids.rs`
  - Newtypes: `ChannelId`, `EventId`, `CursorId`, `EntityId`, `AgentId`, `SessionId` — all `#[repr(transparent)]` over `uuid::Uuid`
  - All derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
  - Verification: `cargo check -p frf-domain` exits 0; no `String` bare IDs

- [ ] **T3** Create `crates/frf-domain/src/envelope.rs`
  - File: `crates/frf-domain/src/envelope.rs`
  - Types: `EventEnvelope`, `Channel`, `Offset`, `Cursor`
  - `EventEnvelope` fields: `id: EventId`, `channel: Channel`, `offset: Offset`, `payload: serde_json::Value`, `timestamp: chrono::DateTime<chrono::Utc>`
  - Public enums get `#[non_exhaustive]`
  - Verification: `cargo check -p frf-domain` exits 0

- [ ] **T4** Create remaining domain type modules
  - Files: `crates/frf-domain/src/entity.rs` (`EntityChange`), `crates/frf-domain/src/agent.rs` (`AgentEvent`), `crates/frf-domain/src/sync.rs` (`SyncOp`), `crates/frf-domain/src/presence.rs` (`Presence`), `crates/frf-domain/src/signal.rs` (`SignalEnvelope`)
  - All public enums `#[non_exhaustive]`; no `unwrap()`/`expect()` in any code
  - Verification: `cargo check -p frf-domain` exits 0

- [ ] **T5** Create `crates/frf-domain/src/lib.rs`
  - File: `crates/frf-domain/src/lib.rs`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Re-exports all public types from submodules
  - Verification: `cargo clippy -p frf-domain -- -D warnings -W clippy::pedantic` exits 0

- [ ] **T6** Create `crates/frf-domain/tests/serde_roundtrip.rs`
  - File: `crates/frf-domain/tests/serde_roundtrip.rs`
  - JSON round-trip tests for every top-level type (serialize → deserialize → assert_eq)
  - Verification: `cargo test -p frf-domain` all tests pass
