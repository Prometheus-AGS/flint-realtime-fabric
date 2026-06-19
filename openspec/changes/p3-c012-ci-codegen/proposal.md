# p3-c012 — Dagger CI codegen pipeline (UniFFI + FRB + proto)

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c009 (Swift SDK), p3-c010 (Kotlin SDK), p3-c011 (Dart SDK)

## Directory
`dagger/`

## What this change does

Extends the existing Dagger CI pipeline to run codegen for all three FFI SDK
bindings. Ensures that SDK stubs are always fresh — any change to `frf-ffi`'s
public surface regenerates Swift, Kotlin, and Dart bindings in CI.

### New Dagger pipeline stages

1. **`rust-build`** — `cargo build -p frf-ffi --release`
2. **`uniffi-swift`** — `cargo run --bin uniffi-bindgen generate --language swift`; diff against committed `frf.swift`; fail if changed
3. **`uniffi-kotlin`** — same for Kotlin `frf.kt`
4. **`frb-dart`** — `flutter_rust_bridge_codegen generate`; diff against committed `frb_generated.dart`; fail if changed
5. **`buf-generate`** — existing proto codegen (carried from Phase 2 debt)
6. **`pnpm-build`** — `pnpm -r build` for TS SDK + entity-management + admin-UI

The pipeline enforces: generated files committed in the repo must match what
`cargo build` + codegen tools produce from the current source. This prevents
drift between the Rust core and the SDK bindings.

### CI fast path

When `crates/frf-ffi/` is unchanged in the commit diff, stages 2–4 are skipped
(Dagger caches by input hash).

## Non-goals

- Does not publish SDKs to registries (Phase 7).
- Does not add iOS/Android hardware test runners.
- Does not run the full E2E suite (that is p3-c013).
