# Verification receipt — pri-c002-local-fixtures

Date: 2026-09-15
Repository: `flint-realtime-fabric`
Base commit: `c5b184535e95e61f3a3ca327d648ced8980cda3c`
Runtime receipt: `20260915084331-29908.json`

## Candidate identity

| Artifact | SHA-256 |
|---|---|
| `compose.integration.yml` | `13ab1dbccd210e21959d7c117fd4cfef9d57e9d61fb06f011a8c024214b7c5ec` |
| `crates/frf-gateway/tests/subscribe_mux.rs` | `7071d864b0cb33bde3bed89616e526a5a60c9dd8e36d5f6be116eccfe55f0d1d` |
| `scripts/run-local-integration.sh` | `f54bd44d3ce83d2b1d54c43e1b8a53f873962b37827f551f3966035d20357988` |
| Iggy image | `iggyrs/iggy@sha256:68a314c1380be5a792a134f3bd346ded42bd49d9f7114c86f70b48fc85bc5272` |

## Deterministic checks

| Check | Result |
|---|---|
| `make local-integration` equivalent with evidence receipt directory and caller `FRF_DISABLE_DELIVERY=1` | PASS; disabled delivery 101, restored delivery 0, scenarios 2/2 |
| Missing-command, zero-scenario and changed-filter probes | PASS; each exited 1 |
| Owned Compose resource query after cleanup | PASS; no containers or volumes remain |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic` | PASS |
| `cargo clippy -p frf-gateway --test subscribe_mux -- -D warnings -W clippy::pedantic` | PASS |
| `cargo test -p frf-gateway --test subscribe_mux` | PASS; required integration test remains explicitly ignored outside the mandatory runner |
| `cargo fmt --check --all` | PASS |
| `shellcheck scripts/run-local-integration.sh` | PASS |
| `bash scripts/check-file-size.sh` | PASS; 271 files, none over 500 lines |
| `git diff --check` | PASS |
| `openspec validate pri-c002-local-fixtures --strict --no-interactive` | PASS |
| Frozen `proto/flint/v1` and inward dependency manifests | PASS; no diff |

## Scope decision

The vacuous Postgres CDC test was removed. It neither inserted a row nor made
an assertion, so retaining it would preserve a false-success surface. Real CDC
commit, rollback, key and checkpoint semantics remain assigned to
`pri-c008-cdc-commit-mapping`; this fixture proves only authenticated gateway
publish/subscription delivery through the real Iggy adapter.

The three pre-existing shape-file hashes still match `entry-state.json`; this
change did not adopt or modify that user-owned work.
