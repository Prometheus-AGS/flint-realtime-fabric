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
| `frb-dart` | `flutter_rust_bridge_codegen generate` | Diff must be empty |
| `buf-generate` | `buf generate` | Must succeed |
| `pnpm-build` | `pnpm -r build` | Must succeed |

Stages `uniffi-swift`, `uniffi-kotlin`, and `frb-dart` are fast when
`crates/frf-ffi/` is unchanged (Dagger caches by input hash).

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
