# p0-c006 — Dagger CI Pipelines

## Affected crates
- None (CI infra only)
- `dagger/` directory (new)

## Dependency-rule impact
Dagger pipelines run outside the Rust workspace; they cannot violate the dependency rule. However, by enforcing `clippy::pedantic` and denying `unwrap`/`expect` in library crates, they become the automated guardian of the dependency rule and quality gates.

## Phase 0 exit criterion satisfied
_Dagger pipelines run on push; all gates green against Phase 0 crates._

## What this change does
1. Creates `dagger/src/main.rs` (Dagger Rust SDK pipeline binary) with four pipeline functions:
   - `fmt_check` — `cargo fmt --all --check`
   - `clippy_gate` — `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
   - `test_gate` — `cargo test --all`
   - `msrv_check` — builds against the pinned MSRV (rust-toolchain.toml minimum)
2. Creates `dagger/Cargo.toml` (standalone binary, `[dependencies]` = dagger-sdk).
3. Creates `rust-toolchain.toml` at workspace root pinning the channel and MSRV.
4. Wires pipeline to `.github/workflows/ci.yml` (or equivalent) via `dagger run`.
5. Each pipeline step is independent; failures report which gate failed.

## Non-goals
- Does not run E2E or integration tests requiring live Iggy / Keto / Kratos (those are Phase 1 CI additions).
- Does not build the admin UI (Phase 4).
- Does not deploy or publish crates.
- Does not include Dagger secret management for external services.
