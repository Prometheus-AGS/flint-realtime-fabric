# Tasks — p1-c001 Workspace Expansion

- [ ] **T1** Add new workspace members to root `Cargo.toml`
  - File: `Cargo.toml`
  - Add under `[workspace] members`: `"crates/frf-app"`, `"crates/frf-broker-iggy"`, `"crates/frf-authz-keto"`, `"crates/frf-identity-ory"`, `"crates/frf-postgres-cdc"`
  - Verification: `cargo check --workspace` exits 0 after stub crates created

- [ ] **T2** Add shared dependencies to `[workspace.dependencies]`
  - File: `Cargo.toml`
  - Add: `async-stream = "0.3"`, `futures-util = "0.3"`, `reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }`, `jsonwebtoken = "9"`, `tokio-postgres = { version = "0.7", features = ["with-serde_json-1"] }`, `async-trait = "0.1"`, `mockall = "0.13"`, `httpmock = "0.7"`
  - Verification: `cargo check --workspace` passes; all new deps resolve

- [ ] **T3** Create stub `crates/frf-app/Cargo.toml` + `src/lib.rs`
  - `Cargo.toml`: workspace package fields, `[dependencies]`: `frf-domain`, `frf-ports`, `thiserror`, `tracing`, `async-trait`, `futures-core`, `futures-util`
  - `src/lib.rs`: `#![deny(warnings)]` + `#![warn(clippy::pedantic)]` + empty module stubs
  - Verification: `cargo check -p frf-app` exits 0

- [ ] **T4** Create stub `crates/frf-broker-iggy/Cargo.toml` + `src/lib.rs`
  - `Cargo.toml`: `[dependencies]`: `frf-domain`, `frf-ports`, `iggy`, `async-stream`, `tokio`, `tokio-stream`, `tracing`, `thiserror`
  - `src/lib.rs`: `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-broker-iggy` exits 0

- [ ] **T5** Create stub `crates/frf-authz-keto/Cargo.toml` + `src/lib.rs`
  - `Cargo.toml`: `[dependencies]`: `frf-domain`, `frf-ports`, `reqwest`, `dashmap`, `serde`, `serde_json`, `tokio`, `tracing`, `thiserror`
  - `src/lib.rs`: `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-authz-keto` exits 0

- [ ] **T6** Create stub `crates/frf-identity-ory/Cargo.toml` + `src/lib.rs`
  - `Cargo.toml`: `[dependencies]`: `frf-domain`, `frf-ports`, `jsonwebtoken`, `reqwest`, `serde`, `serde_json`, `tokio`, `tracing`, `thiserror`
  - `src/lib.rs`: `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-identity-ory` exits 0

- [ ] **T7** Create stub `crates/frf-postgres-cdc/Cargo.toml` + `src/lib.rs`
  - `Cargo.toml`: `[dependencies]`: `frf-domain`, `frf-ports`, `tokio-postgres`, `serde_json`, `tokio`, `tracing`, `thiserror`
  - `src/lib.rs`: `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-postgres-cdc` exits 0

- [ ] **T8** Verify full workspace compiles
  - Command: `cargo check --workspace`
  - Expected: 0 errors across all 9 crates (4 Phase 0 + 5 new stubs)
  - Verification: command exits 0
