# Dagger CI Pipelines

Dagger TypeScript pipelines for Flint Realtime Fabric.

## Pipelines

### `codegen.ts` — FFI SDK + Proto codegen

Ensures generated SDK bindings stay in sync with the Rust source.

| Stage | Tool | Gate |
|-------|------|------|
| `rust-build` | `cargo build -p frf-ffi --release` | Must compile |
| `uniffi-swift` | `uniffi-bindgen generate --language swift` | Diff must be empty |
| `uniffi-kotlin` | `uniffi-bindgen generate --language kotlin` | Diff must be empty |
| `frb-dart` | `flutter_rust_bridge_codegen generate` | **BROKEN — see note below** |
| `buf-generate` | `buf generate` | Must succeed |
| `pnpm-build` | `pnpm -r build` | Must succeed |

Stages `uniffi-swift` and `uniffi-kotlin` are fast when `crates/frf-ffi/` is
unchanged (Dagger caches by input hash).

> **`frb-dart` cannot succeed** (flagged 2026-09-14, p38-c005; not fixed there).
> It runs `flutter_rust_bridge_codegen` against `crates/frf-ffi/src/lib.rs`, a
> UniFFI crate — [ADR-003](../docs/decisions/adr-003-ffi-codegen-versions.md)
> records that FRB's parser panics on `#[uniffi::export]` and that FRB and UniFFI
> cannot co-own the FFI crate. It then diffs against
> `sdks/dart/lib/src/rust/frb_generated.dart`, which does not exist and is
> gitignored. Dart bindings come from `uniffi-bindgen-dart`. Fixing this means
> replacing the stage or deleting it.

```sh
# Run codegen pipeline locally (requires Dagger CLI + Docker)
cd dagger && pnpm install && pnpm codegen
```

## Cargo gates (enforced in CI via GitHub Actions)

| Gate | Command |
|------|---------|
| Format | `cargo fmt --all --check` |
| Lint (pedantic) | `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` |
| Test | `cargo test --all` |
| MSRV | `cargo check --all` on Rust 1.85 |
