# p9-c004 — Dagger Layer 3 E2E Integration Stage

## Summary

Add Dagger Stage 10 (`integration`) gated on `ENABLE_INTEGRATION_STAGE=true`.
Starts the Compose stack, waits for gateway healthz, runs Playwright E2E with
`WASM_AVAILABLE=1`, then tears down.

## Files to Modify

- `dagger/codegen.ts` — add Stage 10 with Docker-compose integration flow

## Design Note

Full Docker-in-Docker requires the CI host to expose `/var/run/docker.sock`.
The Dagger stage documents this requirement via comments. The Compose stack
already exists at `compose.yml`.

## Exit Criteria

- `dagger/codegen.ts` typechecks
- Stage is guarded on `ENABLE_INTEGRATION_STAGE=true`
- `docker compose config` exits 0 (existing Compose file valid)
