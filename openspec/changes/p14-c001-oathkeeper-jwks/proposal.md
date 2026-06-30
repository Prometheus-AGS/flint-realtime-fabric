# p14-c001 — Replace Oathkeeper with flint-gate in compose stack

> Phase: phase-14-stage10-dind-live-triage · Priority: CRITICAL
> **Corrected:** original proposal incorrectly targeted oathkeeper. flint-gate
> is the exclusive auth proxy — Oathkeeper is not used in this project.

## Problem

`compose.yml` had an `oathkeeper` service and `GATEWAY_JWKS_URL` env var.
Oathkeeper is not used — flint-gate replaces it entirely. Without flint-gate
in the compose stack, there is no auth proxy and no JWKS source for the gateway.

Additionally, `GATEWAY_JWKS_URL` in gateway config/env was a misnomer that
leaked Oathkeeper's name into code that has nothing to do with Oathkeeper.

## Solution

1. Replace the `oathkeeper` service in `compose.yml` with a `flint-gate` service
   built from `/Users/gqadonis/Projects/prometheus/flint-gate`.
2. Create `deploy/flint-gate/config.yaml` — dev config wiring flint-gate as
   a passthrough proxy in front of `gateway:8080`, minting JWTs for the gateway.
3. Rename `GATEWAY_JWKS_URL` → `GATEWAY_JWKS_URL` in:
   - `crates/frf-gateway/src/config.rs` (field + env var read)
   - `crates/frf-gateway/src/main.rs` (field reference)
   - `crates/frf-gateway/tests/subscribe_mux.rs` (comment)
   - `compose.yml` (env value)
4. Point `GATEWAY_JWKS_URL` at flint-gate's admin signing-keys endpoint:
   `http://flint-gate:4457/signing-keys`

## Files Changed

- `compose.yml` — replace oathkeeper service with flint-gate; rename env var
- `deploy/flint-gate/config.yaml` — NEW: dev flint-gate config
- `crates/frf-gateway/src/config.rs` — rename field + env var
- `crates/frf-gateway/src/main.rs` — rename field reference
- `crates/frf-gateway/tests/subscribe_mux.rs` — update comment
- `deploy/flint-gate/jwks.json` — REMOVED (was incorrectly created)

## Status

DONE — all file changes applied.

## Acceptance Criteria

- [ ] No `oathkeeper` references remain in compose.yml
- [ ] `GATEWAY_JWKS_URL` env var used everywhere (no OATHKEEPER_)
- [ ] `deploy/flint-gate/config.yaml` exists
- [ ] `cargo check -p frf-gateway` passes
