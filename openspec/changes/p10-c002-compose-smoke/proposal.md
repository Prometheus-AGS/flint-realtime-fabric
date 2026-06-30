# p10-c002 — Compose Smoke Validation

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Verify `compose.yml` brings all services to healthy state locally; fix any
startup blockers (missing configs, missing `Dockerfile`, env vars); confirm
Dagger Stage 10 integration smoke gate can run.

## Files to Create/Modify

- `Dockerfile` — confirm exists at repo root; create minimal multi-stage build
  if missing (copy `target/release/frf-gateway` into `debian:bookworm-slim`)
- `deploy/oathkeeper/config.yml` — confirm present; create stub if missing
- `deploy/keto/keto.yml` — confirm present; create stub if missing
- `compose.yml` — fix any missing service config references

## Design Notes

The `compose.yml` already references `build: context: . dockerfile: Dockerfile`
for the gateway service. If the `Dockerfile` is absent the compose up will fail
immediately with a build error.

The Dagger Stage 10 smoke check (`ENABLE_INTEGRATION_STAGE=true`) runs
`docker compose up -d`, polls `http://localhost:8080/healthz` for 60s, then
runs Playwright Layer 2 tests. This change validates the compose file is
actually startable before Stage 10 wires to it in CI.

## Exit Criteria

- `docker compose config` exits 0 (compose file is syntactically valid)
- `docker compose up -d` exits 0 locally with all 7 services starting
- `curl http://localhost:8080/healthz` returns 200 within 60s
- `docker compose down` exits 0 cleanly
- No service exits non-zero during the smoke run
