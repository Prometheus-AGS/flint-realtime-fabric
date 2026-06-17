# Dagger CI Pipelines

This directory is the future home of Dagger SDK pipeline scripts for FRF.

## Current state

CI gates run via `.github/workflows/ci.yml` (GitHub Actions) using direct
`cargo` invocations. Migration to Dagger SDK pipelines is planned for Phase 1.

## Gates (enforced in CI)

| Gate | Command |
|------|---------|
| Format | `cargo fmt --all --check` |
| Lint (pedantic) | `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` |
| Test | `cargo test --all` |
| MSRV | `cargo check --all` on Rust 1.85 |

All four gates must be green before merging to `main`.
