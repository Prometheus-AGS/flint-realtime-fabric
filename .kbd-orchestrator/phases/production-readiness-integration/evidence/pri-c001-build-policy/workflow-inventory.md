# Remote execution inventory — pri-c001-build-policy

Captured: 2026-09-15
Source base: `3043ca53239cc74a7f94a9e12484e2fd17726adb`

## GitHub Actions

| File | Remotely reachable operations | Classification |
|------|-------------------------------|----------------|
| `.github/workflows/ci.yml` | `cargo fmt --check`, file-size lint, Clippy lint, locked workspace check on Rust 1.94 | format, lint, build/typecheck |
| `.github/workflows/docs-pages.yml` | locked dependency install, Docusaurus build/link validation, output sanitization, Pages packaging/deployment | build, validation, packaging |
| `.github/workflows/publish-candidate.yml` | immutable container build with provenance/SBOM and GHCR publication | build, packaging |

The former `ci.yml` `cargo test --all` job and the complete
`decode-proof.yml` browser/runtime workflow were removed. No remaining workflow
runs Rust, browser, integration, decode, or benchmark tests.

## Dagger

`dagger/codegen.ts` now materializes only these paths:

1. production/test-target Clippy lint;
2. release build of `frf-ffi`;
3. Swift and Kotlin UniFFI generation plus diff validation;
4. protobuf generation;
5. WASM build, output validation, and size packaging gate;
6. admin UI lint and recursive package build/typecheck.

The Playwright smoke stage, opt-in Criterion benchmark, and opt-in Compose plus
Playwright integration stage were removed. Dart generation is also absent: the
accepted ADR-003 generator is `uniffi-bindgen-dart`, whose current output needs
the compatibility patches documented in `sdks/dart/GENERATED.md`.

## Preserved local runtime entry points

| Removed remote path | Local command | Owning readiness change |
|---------------------|---------------|-------------------------|
| Rust test job | `cargo test --workspace --locked` | `pri-c023-release-signoff` |
| Dagger Playwright smoke | `pnpm --dir admin-ui exec playwright test e2e/ --reporter=list` | `pri-c021-admin-auth`, `pri-c022-platform-parity` |
| Dagger Layer 3 integration | `make layer3-e2e` | `pri-c022-platform-parity` |
| Decode workflow | `./scripts/run-media-decode.sh` | `pri-c020-sovereign-decode` |
| Dagger Criterion benchmark | `cargo bench -p frf-crdt --bench crdt_merge -- --baseline main && bash scripts/bench-regression-check.sh` | `pri-c023-release-signoff` |

The owning changes must execute these commands locally against their required
fixtures. This inventory only proves that remote runtime execution has been
removed and that operator entry points remain available.
