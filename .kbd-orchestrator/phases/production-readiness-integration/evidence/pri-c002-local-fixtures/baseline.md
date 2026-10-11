# Baseline — pri-c002-local-fixtures

Captured: 2026-09-15
Fabric commit: `c5b184535e95e61f3a3ca327d648ced8980cda3c`

## Dependency receipt

`pri-c001-build-policy` is archived and its source-bound receipt plus isolated
cross-model PASS review are present. Canonical KBD status is `DONE` with
implementation `COMPLETE`.

## Affected-path fingerprints

| Path | SHA-256 / state |
|------|----------------|
| `compose.yml` | `a58fb429583493ea86c54d4e41741a4371b367c84190b92f3384e4f9ecda5fc9` |
| `compose.ci.yml` | `1db6dae706b9b8ffa4bbd3455b9c2c28d247240a8e49d0a692f43244134a29a5` |
| `crates/frf-gateway/tests/subscribe_mux.rs` | `58d1b6742f81f833ac88d73a498e5f2317faabdd8b6a52e9511f644f891ba865` |
| `crates/frf-postgres-cdc/tests/cdc_integration.rs` | `7f31a88cbac2ef6101385a4aa7d8f207c4de46cd2b4f36f836d0eeb52af12abd` |
| `scripts/` | existing local helpers; no namespaced general integration runner |

These paths had no unstaged changes at entry to c002. The separately recorded
shape-facade user edits remain outside this change.

## Demonstrated defects

- `subscribe_mux.rs` is an ignored 32-line outline with no assertion and no
  gateway, subscriber, or publisher construction.
- `cdc_integration.rs` never inserts a row and only prints a possibly zero
  publish count. CDC commit semantics remain assigned to c008.
- Existing Compose files have fixed project/service/volume identity and no
  run-owned receipt or cleanup contract.
- No runner rejects missing prerequisites or a zero/skip-only scenario count.
