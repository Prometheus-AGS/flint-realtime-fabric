# Tasks — p0-c001 Workspace Restructure

- [x] **T1** Replace `Cargo.toml` with virtual workspace manifest
  - File: `Cargo.toml`
  - Verification: `cargo metadata --no-deps` → workspace_root OK, 0 members

- [x] **T2** Add `[workspace.package]` block
  - Fields: `version = "0.1.0"`, `edition = "2024"`, `license = "MIT"`, `repository = "https://github.com/prometheusags/flint-realtime-fabric"`, `rust-version = "1.82"`
  - Verification: `cargo metadata --no-deps` parses without error ✓

- [x] **T3** Add `[workspace.dependencies]` with exact pinned versions
  - Includes: `axum = "0.8.8"`, `tokio = "1.52"`, `tonic = "0.14"`, `prost = "0.14"`, `ractor = "0.15"`, `serde = "1.0"`, `serde_json = "1.0"`, `bytes = "1.11"`, `thiserror = "2.0"`, `anyhow = "1.0"`, `tracing = "0.1"`, `tracing-subscriber = "0.3"`, `uuid = "1.23"`, `chrono = "0.4"`, `dashmap = "6"`
  - Iggy fork: `iggy = { git = "https://github.com/GQAdonis/iggy", branch = "master" }`
  - Verification: `cargo metadata --no-deps` ✓

- [x] **T4** Add `[profile.release]` block
  - `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`
  - Verification: manifest parses ✓

- [x] **T5** Delete `src/main.rs`
  - Verification: `ls` shows no `src/` directory ✓

- [x] **T6** Create `rust-toolchain.toml` at workspace root
  - Pins `channel = "stable"`, components `rustfmt` + `clippy`, targets for aarch64/x86_64-linux-musl/wasm32
  - Verification: `rustup` synced to 1.96.0 stable ✓
