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
| `buf-generate` | `buf generate` | Must succeed |
| `wasm-build` | `wasm-pack build` | Must compile and meet the size limit |
| `pnpm-build` | `pnpm -r build` | Must succeed |

Stages `uniffi-swift` and `uniffi-kotlin` are fast when `crates/frf-ffi/` is
unchanged (Dagger caches by input hash).

The pipeline does not regenerate Dart. [ADR-003](../docs/decisions/adr-003-ffi-codegen-versions.md)
requires `uniffi-bindgen-dart`, and its current 0.1.3 output needs the manual,
documented compatibility patches in [`sdks/dart/GENERATED.md`](../sdks/dart/GENERATED.md).
Run `./sdks/dart/build_dart.sh` locally when changing the FFI surface and review
the regenerated output before restoring those patches.

```sh
# Run codegen pipeline locally (requires Dagger CLI + Docker)
cd dagger && pnpm install && pnpm codegen
```

## Remote quality gates

| Gate | Command |
|------|---------|
| Format | `cargo fmt --all --check` |
| Lint (pedantic) | `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` |
| MSRV | `cargo check --workspace --locked` on Rust 1.94 |

GitHub Actions and Dagger run build, lint, typecheck, format, code generation,
and packaging only. Runtime checks remain direct local commands:

| Local gate | Command | Owning readiness change |
|------------|---------|-------------------------|
| Rust tests | `cargo test --workspace --locked` | `pri-c023-release-signoff` |
| Browser smoke | `pnpm --dir admin-ui exec playwright test e2e/ --reporter=list` | `pri-c021-admin-auth` / `pri-c022-platform-parity` |
| Layer 3 stack | `make layer3-e2e` | `pri-c022-platform-parity` |
| Sovereign decode | `./scripts/run-media-decode.sh` | `pri-c020-sovereign-decode` |
| CRDT benchmark | `cargo bench -p frf-crdt --bench crdt_merge -- --baseline main && bash scripts/bench-regression-check.sh` | `pri-c023-release-signoff` |
