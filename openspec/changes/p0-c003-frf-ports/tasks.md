# Tasks — p0-c003 frf-ports

**Semver note:** Trait signature changes in this crate are semver major for all adapter crates. Treat signatures as frozen once Phase 0 is approved.

- [ ] **T1** Create `crates/frf-ports/Cargo.toml`
  - File: `crates/frf-ports/Cargo.toml`
  - `[dependencies]`: `frf-domain = { path = "../frf-domain" }`, `async-trait`, `thiserror = { workspace = true }`, `bytes = { workspace = true }`, `tracing = { workspace = true }`
  - Add to `[workspace.members]` in root `Cargo.toml`
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T2** Create `crates/frf-ports/src/log_broker.rs`
  - File: `crates/frf-ports/src/log_broker.rs`
  - `LogBroker` trait with methods: `publish`, `subscribe`, `seek`, `ack`
  - Associated `Error` type bounded `std::error::Error + Send + Sync + 'static`
  - `#[async_trait]` (or native async-in-trait if MSRV allows `return_position_impl_trait_in_trait`)
  - `#[tracing::instrument(name = "port::LogBroker::publish", ...)]` on each method
  - No implementations — trait definition only
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T3** Create remaining port trait modules
  - Files: `crates/frf-ports/src/authz.rs` (`AuthzProvider`), `crates/frf-ports/src/identity.rs` (`IdentityVerifier`), `crates/frf-ports/src/crdt_store.rs` (`CrdtStore`), `crates/frf-ports/src/media.rs` (`MediaSignaler`), `crates/frf-ports/src/federation.rs` (`FederationBridge`)
  - Each trait has `tracing::instrument` on all methods
  - No `unwrap()`/`expect()` anywhere; all errors through associated `Error` type
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T4** Create `crates/frf-ports/src/error.rs`
  - File: `crates/frf-ports/src/error.rs`
  - Shared `PortError` enum variants covering transport, authz, not-found, timeout
  - `#[derive(thiserror::Error, Debug)]`
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T5** Create `crates/frf-ports/src/lib.rs`
  - File: `crates/frf-ports/src/lib.rs`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Re-exports all port traits and `PortError`
  - Verification: `cargo clippy -p frf-ports -- -D warnings -W clippy::pedantic` exits 0

- [ ] **T6** Verify no adapter crate appears in frf-ports dependencies
  - Check: `cargo tree -p frf-ports` contains no `frf-broker-*`, `frf-authz-*`, or `frf-gateway`
  - Verification: manual inspection + `cargo metadata -p frf-ports --no-deps` shows only domain + utility deps
