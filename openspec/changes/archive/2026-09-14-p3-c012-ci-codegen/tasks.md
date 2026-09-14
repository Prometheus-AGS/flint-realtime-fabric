# Tasks — p3-c012 ci-codegen

- [ ] **T1** Read existing `dagger/` pipeline files to understand current structure
  - `ls dagger/`; read main pipeline file
  - Determine if Dagger pipelines are in Go or TypeScript
  - Verification: existing pipeline understood; no changes yet

- [ ] **T2** Add `rust-build-ffi` stage to Dagger pipeline
  - Stage: `cargo build -p frf-ffi --release` in the Rust workspace container
  - Output artifact: `target/release/libfrf_ffi.dylib` (macOS) or `.so` (Linux CI)
  - Mount from stage output to subsequent stages
  - Verification: `dagger run` with only this stage exits 0

- [ ] **T3** Add `uniffi-swift` stage
  - Depends on `rust-build-ffi` output
  - Run: `cargo run --bin uniffi-bindgen generate --library ... --language swift --out-dir /tmp/swift-gen`
  - Diff `/tmp/swift-gen/frf.swift` against committed `sdks/swift/Sources/FrfClient/frf.swift`
  - If diff non-empty: print diff and exit 1 ("Swift bindings out of date — run ./sdks/swift rebuild")
  - Verification: pipeline stage exits 0 when committed bindings match

- [ ] **T4** Add `uniffi-kotlin` stage
  - Same pattern as T3 but for Kotlin
  - Diff against committed `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt`
  - Verification: pipeline stage exits 0 when committed bindings match

- [ ] **T5** Add `frb-dart` stage
  - Depends on `rust-build-ffi` output
  - Run: `flutter_rust_bridge_codegen generate ...`; diff against committed `sdks/dart/lib/src/rust/frb_generated.dart`
  - Verification: pipeline stage exits 0 when committed bindings match

- [ ] **T6** Add `buf-generate` stage (closes Phase 2 CI gap)
  - Run `buf generate` from `proto/` to regenerate Go, TS, C# SDK stubs
  - Diff against committed `sdks/go/`, `sdks/ts/src/gen/`, `sdks/csharp/`
  - Verification: exits 0 when committed stubs match

- [ ] **T7** Add `pnpm-build` stage
  - Run `pnpm install && pnpm -r build` in Node container
  - Verification: exits 0; confirms admin-ui and SDK builds work

- [ ] **T8** Wire all stages into the main CI entrypoint
  - The codegen stages run in parallel (they don't depend on each other, only on rust-build-ffi)
  - `pnpm-build` runs after `buf-generate` (uses generated TS SDK)
  - Verification: full `dagger run` exits 0 (no live SurrealDB/Iggy required for codegen stages)
