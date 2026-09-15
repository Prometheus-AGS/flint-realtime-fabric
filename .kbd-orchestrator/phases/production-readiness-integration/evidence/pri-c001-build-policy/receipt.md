# Source-bound receipt — pri-c001-build-policy

Captured: 2026-09-15
Repository: `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric`
Branch: `codex/production-readiness-integration`
Base commit: `3043ca53239cc74a7f94a9e12484e2fd17726adb`

## Candidate source fingerprints

| Path | SHA-256 |
|------|---------|
| `Cargo.toml` | `03d5e7d01c92ac588d32c99b056c6cb3059eef2c2bd187c56ec77a2a52bb2010` |
| `.github/workflows/ci.yml` | `319f4da27511dc9fc7090681ac2b0c54f596a523640f69bc2c5fff5756eeeaab` |
| `compose.ci.yml` | `1db6dae706b9b8ffa4bbd3455b9c2c28d247240a8e49d0a692f43244134a29a5` |
| `crates/frf-gateway/src/routes/dev.rs` | `76c0ff2b42c428d1b9eea732f867d29fc3196558cb2e2537c6a368c1558853a3` |
| `dagger/codegen.ts` | `c05785de906da039037048346be37ab9f8eab0385cbf0fb7061218f6f9fa5f7d` |
| `dagger/README.md` | `8b77ae13202857eca7fc7f4bb17a609e30ba57acc93f2f5966cf70bd5037e3ce` |
| `dagger/tsconfig.json` | `5f8e7d20a1631e3aba6f13ec27405ceed68603f0976d124e07ea628ff77e2c27` |
| `Makefile` | `4ae85457b78126cc3c35f08ed37e8cf8c098f67beebce18ad6dcdad4380eae08` |
| `scripts/run-layer3-e2e.sh` | `d314208d0e3aca314996c305b2740a35c65bb85423a72cfe91e8b0d09b471383` |
| `README.md` | `28911ae5f54c3a2cdae32e6b142c2591faa843d18fc500c1191cb73d0d4f8100` |
| `docs/DEVELOPMENT.md` | `450434d9c058796c379bc57684bd18eb37fc9c7f9ae8ca548534f937b0ed6f66` |

The deleted `.github/workflows/decode-proof.yml` is represented by its deletion
from the base commit. The three shape-facade files identified in
`entry-state.json` as pre-existing user work are outside this change's
`files.txt` and review scope.

## Local verification

All commands ran from the Fabric repository unless a working directory is
shown. Every listed command exited 0 on the candidate above.

| Gate | Command | Result |
|------|---------|--------|
| Default workspace | `cargo check --workspace --locked` | PASS |
| Dev endpoints | `cargo check -p frf-gateway --features dev-endpoints --locked` | PASS |
| Gateway shape facade | `cargo check -p frf-gateway --features shape-facade --locked` | PASS |
| Rust SDK shape facade | `cargo check -p frf-sdk-rust --features shape-facade --locked` | PASS |
| Production lint | `cargo clippy --workspace --lib --bins --locked -- -D warnings -W clippy::pedantic` | PASS |
| Dev endpoint lint | `cargo clippy --workspace --lib --bins --features frf-gateway/dev-endpoints --locked -- -D warnings -W clippy::pedantic` | PASS |
| Test-target lint | `cargo clippy --workspace --tests --locked -- -D warnings -W clippy::pedantic` | PASS |
| Format | `cargo fmt --all --check` | PASS |
| File size | `bash scripts/check-file-size.sh` | PASS, 271 files checked and none above 500 lines |
| Package build/typecheck | `pnpm install --frozen-lockfile && pnpm -r build` | PASS, three workspace projects |
| Dagger TypeScript | `(cd dagger && npm ci && npm exec -- tsc --noEmit --pretty false)` | PASS after adding the required DOM library |
| Shell syntax | `bash -n scripts/run-layer3-e2e.sh` | PASS |
| Compose rendering | `docker compose -f compose.ci.yml config` | PASS |
| OpenSpec | `openspec validate pri-c001-build-policy --strict --no-interactive` | PASS |
| Remote runtime inventory | negative scan over `.github/workflows`, `dagger/codegen.ts`, and `dagger/package.json` | PASS; no runtime test or benchmark invocation remains |

The first post-review Cargo rerun detected a corrupt local Iggy git cache, not a
source failure. The corrupt cache was moved intact to
`/tmp/cargo-iggy-cache-corrupt-20260915T080933Z`; `cargo fetch --locked`
restored revision `d34b9c96ad5a15334e06040d68fd7512beeba4c8`, after which the complete
Cargo check and Clippy matrix passed.

## QA and review

- Artifact-refiner state: `.refiner/artifacts/pri-c001-build-policy/`
- Artifact-refiner result: PASS, zero blocking constraint violations.
- Initial cross-model adversarial review: BLOCK with three critical findings.
- Corrections: explicitly materialized `rustBuild`, put the already authoritative
  Rust 1.94 pin into the diff, and added this source-bound receipt.
- Corrected-candidate adversarial review: PASS with zero critical findings, two
  warnings, and zero suggestions. The cross-model check is verified distinct
  (`gpt-5` producer, `gpt-5.5` judge through the isolated REST gateway).
- Final findings SHA-256:
  `c82f60ad5c99272d5fc9b698975e190d63459f2aab4f509595bf6aad46b60c14`.
- Nonblocking warnings retained in the review receipt: the remote Clippy table
  abbreviates the actual three-command matrix, and the compile-only synthetic
  `subject: None` repair has no route-level test. The latter is covered by the
  owning local integration changes; neither warning weakens a blocking gate.
