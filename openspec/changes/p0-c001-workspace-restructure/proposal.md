# p0-c001 — Workspace Restructure

## Affected crates
- Root `Cargo.toml` (replaced, not a crate)
- `src/main.rs` (deleted)
- Future members: `crates/frf-domain`, `crates/frf-ports`, `crates/frf-app`, `crates/frf-gateway`, `crates/frf-proto`, and all adapter crates

## Dependency-rule impact
This is the foundational scaffold. No business-logic layers are touched. The workspace manifest establishes `[workspace.dependencies]` that every member crate inherits, enforcing the single-version-pinning invariant. No adapter crates can exist yet, so the compiler dependency rule cannot be violated.

## Phase 0 exit criterion satisfied
_Workspace compiles_ — `cargo check --workspace` passes after this change with an empty member list; subsequent changes add members.

## What this change does
1. Replaces `Cargo.toml` with a virtual workspace manifest (`[workspace]`, resolver = "2", edition 2024).
2. Adds `[workspace.package]` block: version, license, repository, edition.
3. Adds `[workspace.dependencies]` pinning the shared stack: `tokio`, `axum = "0.8.8"`, `tonic`, `prost`, `ractor`, `serde`, `serde_json`, `bytes`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `uuid`, `chrono`, `dashmap`, and the Iggy fork `iggy = { git = "https://github.com/GQAdonis/iggy", branch = "master" }`.
4. Deletes `src/main.rs` (hello-world binary — workspace root is not a crate).
5. Adds `[profile.release]` with `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`.

## Non-goals
- Does not create any member crates (those are p0-c002 through p0-c006).
- Does not configure Dagger CI (p0-c006).
- Does not add proto files or FFI crates.
- Does not add any Cargo features or target-specific dependencies.
