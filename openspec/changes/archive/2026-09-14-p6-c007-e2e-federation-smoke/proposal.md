# p6-c007 — Phase 6 E2E Smoke Test

## Summary

Write `admin-ui/e2e/phase6-smoke.spec.ts` validating the federation layer
following the 3-layer Playwright pattern established in phases 4 and 5.

## Phase 6 Exit Criterion

A Matrix room event ingested via the `MatrixBridge` appears in the admin-ui
entity stream table, confirmed by a Playwright E2E test.

## Test Layers

### Layer 1 — UI (no gateway required)

These tests run against the Vite dev server only (`SKIP_INTEGRATION=true`).

1. Federation status section renders (heading present)
2. "No federation events" empty state is shown when store is empty
3. Matrix protocol badge renders when a synthetic event is injected via
   `window.__frf_dev.agentEventStore` (or a federation store if added)
4. ATProto protocol badge renders with synthetic event
5. Navigation to `#federation` route returns 200 (if route added in this phase)

### Layer 2 — WebSocket / gRPC lifecycle (requires gateway on 8080 + 9090)

Gated by `SKIP_INTEGRATION` env var.

6. gRPC `RunAgent` endpoint is reachable on port 9090 (health check via
   `grpc_health_v1` or a test proto call)
7. Federation bridge background task starts without error (gateway logs checked
   via stdout capture in test setup if possible; otherwise skip with `test.skip`)

### Layer 3 — Bus end-to-end (requires full stack with bridges)

Gated by `SKIP_INTEGRATION` env var.

8. Publish a mock Matrix event to the `MatrixBridge` via test HTTP endpoint
   (or use `MockMatrixClient` fixture exposed under dev-only route `/dev/inject-matrix`)
9. Event appears in admin-ui entity stream within 2000ms (polling `page.evaluate`)
10. Protocol label shows "Matrix"

## Dev-only Injection Endpoint (gateway)

To enable Layer 3 without a real Tuwunel homeserver, add a dev-only HTTP route
`POST /dev/inject-federation-event` to `frf-gateway` that takes a JSON body
and injects it directly into the federation bridge pipeline. This route MUST
be gated by `#[cfg(debug_assertions)]` and must return `404` in release builds.

## Files Affected

- `admin-ui/e2e/phase6-smoke.spec.ts` (NEW)
- `crates/frf-gateway/src/routes/dev.rs` (NEW — dev-only injection endpoint)
- `crates/frf-gateway/src/lib.rs` (MODIFY — register dev route under `#[cfg(debug_assertions)]`)

## Quality Gates

- [ ] All Layer 1 tests (5 total) pass with `SKIP_INTEGRATION=true`
- [ ] Layer 2 + 3 tests are properly gated and skip cleanly when `SKIP_INTEGRATION=true`
- [ ] Dev injection endpoint returns `404` in release builds (verified by test)
- [ ] Test file follows exact pattern from `phase5-smoke.spec.ts`
- [ ] No flaky timeout-based assertions — use `waitForSelector` or polling with `expect.poll`
- [ ] `pnpm typecheck` in `admin-ui/` passes
