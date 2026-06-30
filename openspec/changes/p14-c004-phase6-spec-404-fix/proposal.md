# p14-c004 — Fix Phase 6 "404 in release" test assertion

> Phase: phase-14-stage10-dind-live-triage · Priority: HIGH

## Problem

Phase 6 Layer 1 has a test: "dev injection endpoint returns 404 in release builds (or is absent)". The test only runs when `SKIP_INTEGRATION=false` (Stage 10). But Stage 10 builds the gateway WITH the `dev-endpoints` feature, so the endpoint returns `202 Accepted`, not `404`. The test fails with `expected 404, got 202`.

The test was designed to verify production builds omit the endpoint. That invariant is enforced by the Cargo feature flag — the test is checking the wrong condition for Stage 10.

## Solution

Add a `DEV_ENDPOINTS_ENABLED` env var check so the test skips when the compose gateway enables dev endpoints, or change the assertion to accept both 202 and 404.

Preferred: change the test to either:

**Option A** — skip when dev endpoints are enabled (cleaner semantics):

```typescript
test("dev injection endpoint absent in production builds", async ({ request }) => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to verify dev endpoint");
  test.skip(
    process.env["DEV_ENDPOINTS_ENABLED"] === "true",
    "compose build enables dev-endpoints — endpoint present by design"
  );
  const resp = await request.get(`${GATEWAY_URL}/dev/inject-federation-event`);
  expect(resp.status()).toBe(404);
});
```

**Option B** — assert endpoint is either absent or present (less strict):

```typescript
expect([202, 404]).toContain(resp.status());
```

Choose Option A — it preserves the test's original purpose while correctly gating it.

Add to `dagger/codegen.ts` Stage 10 env:

```typescript
.withEnvVariable("DEV_ENDPOINTS_ENABLED", "true")
```

## Files Changed

- `admin-ui/e2e/phase6-smoke.spec.ts` — add `DEV_ENDPOINTS_ENABLED` skip guard to "404 in release" test
- `dagger/codegen.ts` — add `DEV_ENDPOINTS_ENABLED=true` to Stage 10 container env

## Acceptance Criteria

- [ ] The "404" test skips when `DEV_ENDPOINTS_ENABLED=true`
- [ ] The test still runs when `DEV_ENDPOINTS_ENABLED` is unset (production-equivalent)
- [ ] Stage 10 sets `DEV_ENDPOINTS_ENABLED=true` in the test container env
- [ ] `pnpm typecheck` passes
