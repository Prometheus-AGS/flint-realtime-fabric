# p11-c003 — Layer 3 E2E: Dagger Stage 10 Fix + Full-Stack Run

## Phase
phase-11-layer3-e2e-wasm-opt-cdc

## Summary

Fix two bugs in Dagger Stage 10 (Layer 3 E2E):
1. Pass `GATEWAY_URL=http://localhost:28080` to Playwright so Layer 2 tests
   are not skipped inside the Dagger container.
2. Pass `SKIP_INTEGRATION=false` explicitly to enable Layer 2 test execution.

Then verify that `ENABLE_INTEGRATION_STAGE=true` Dagger Stage 10 runs without
fatal errors (individual Layer 3 test failures are acceptable if documented).

## Files to Create/Modify

- `dagger/codegen.ts` — Stage 10: add env var injection before `playwright test`:
  ```typescript
  .withEnvVariable("GATEWAY_URL", "http://localhost:28080")
  .withEnvVariable("SKIP_INTEGRATION", "false")
  ```
  (Already has `WASM_AVAILABLE=1` from previous phases)

## Design Notes

Inside the Dagger DinD runner, the compose gateway is exposed on the Docker
host at `localhost:28080` (mapped from container port 8080). The `GATEWAY_URL`
must match this remapped port — the current Stage 10 healthz poll already uses
the correct port for the health check, but Playwright has no `GATEWAY_URL` set.

The `SKIP_INTEGRATION` env var must be set to `"false"` (not just omitted)
because the layer2 specs check:
```typescript
const skipIntegration = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
```
Without `GATEWAY_URL` set, both conditions trigger a skip. Setting both vars
ensures all Layer 2 tests attempt to run.

Layer 3 tests (`phase4-smoke.spec.ts` CDC, `phase5-smoke.spec.ts` agent bus,
`phase6-smoke.spec.ts` federation) may still fail due to missing iggy topic
pre-creation or CDC consumer startup timing — failures should be documented,
not silently skipped.

## Exit Criteria

- `dagger/codegen.ts` passes TypeScript typecheck after edit
- Stage 10 `env` block includes `GATEWAY_URL` and `SKIP_INTEGRATION=false`
- `ENABLE_INTEGRATION_STAGE=true npx dagger run` completes (exit 0 or
  documented failures in Layer 3 tests)
- Layer 2 tests are NOT skipped inside Stage 10 (verified via playwright output)
