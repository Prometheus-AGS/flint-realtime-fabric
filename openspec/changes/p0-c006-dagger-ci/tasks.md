# Tasks — p0-c006 Dagger CI Pipelines

- [ ] **T1** Create `rust-toolchain.toml` at workspace root
  - File: `rust-toolchain.toml`
  - `[toolchain]` with `channel = "stable"` and `components = ["rustfmt", "clippy"]`
  - Verification: `rustup show active-toolchain` uses the pinned channel

- [ ] **T2** Create `dagger/Cargo.toml`
  - File: `dagger/Cargo.toml`
  - Standalone binary (not a workspace member; Dagger SDK has its own resolver)
  - `[dependencies]`: `dagger-sdk` at latest stable version
  - Verification: `cargo check --manifest-path dagger/Cargo.toml` exits 0

- [ ] **T3** Create `dagger/src/main.rs` with four pipeline functions
  - File: `dagger/src/main.rs`
  - `fmt_check()` — `cargo fmt --all --check`
  - `clippy_gate()` — `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
  - `test_gate()` — `cargo test --all`
  - `msrv_check()` — builds against MSRV from `rust-toolchain.toml`
  - Each returns a `Result<(), anyhow::Error>`; main runs all four sequentially and exits non-zero on first failure
  - Verification: `dagger run cargo run --manifest-path dagger/Cargo.toml` on Phase 0 workspace exits 0

- [ ] **T4** Create `.github/workflows/ci.yml`
  - File: `.github/workflows/ci.yml`
  - Triggers on `push` and `pull_request` to `main`
  - Single job: checks out code, installs Dagger, runs `dagger run cargo run --manifest-path dagger/Cargo.toml`
  - Verification: workflow file validates via `actionlint` (or equivalent); manual review confirms triggers

- [ ] **T5** Verify clippy::pedantic passes on all Phase 0 crates
  - Command: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
  - Expected: 0 errors, 0 warnings on `frf-domain`, `frf-ports`, `frf-proto`, `frf-gateway`
  - Verification: command exits 0

- [ ] **T6** Verify all Phase 0 tests pass under Dagger
  - Command: `dagger run cargo run --manifest-path dagger/Cargo.toml` (runs test_gate)
  - Expected: all unit and integration tests in Phase 0 crates pass
  - Verification: `dagger run` exits 0
