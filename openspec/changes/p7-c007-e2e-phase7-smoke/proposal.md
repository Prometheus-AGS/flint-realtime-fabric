# p7-c007 — Phase 7 E2E Smoke Test

## Summary

Write `admin-ui/e2e/phase7-smoke.spec.ts` using the 3-layer pattern.
Layer 1 verifies static UI shapes and WASM import graceful fallback.
Layers 2 and 3 are gated by `SKIP_INTEGRATION`.

## Changes

### `admin-ui/e2e/phase7-smoke.spec.ts` (NEW)

```typescript
/**
 * Phase 7 exit-criterion smoke tests.
 * 
 * Layer 1 — UI shape (no live gateway required)
 * Layer 2 — gRPC bidi + str0m healthcheck (SKIP_INTEGRATION gate)
 * Layer 3 — WASM AgentStream subscription end-to-end (SKIP_INTEGRATION gate)
 */
import { test, expect } from "@playwright/test";

const SKIP_INTEGRATION = !!process.env["SKIP_INTEGRATION"] || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:8080";
```

**Layer 1 tests (4 tests):**
1. `WebRtcPanel renders in signaling tab` — navigate to `/#signaling`, verify `WebRtcPanel` heading visible
2. `Agent activity panel shows connection mode badge` — navigate to `/#agents`, verify mode badge ("WASM" or "WS") is visible
3. `WASM import resolves gracefully when SDK not built` — evaluate `window.__frf_dev?.wasmAvailable` — either true or false, but no uncaught exception
4. `Dev inject endpoint returns 202 with payload` — skip if `SKIP_INTEGRATION`; POST with valid body → expect 202

**Layer 2 tests (2 tests, gated):**
1. `RunAgent bidi gRPC endpoint responds on port 9090`
2. `Gateway /healthz is up after str0m signaler init`

**Layer 3 tests (2 tests, gated):**
1. `WASM AgentStream receives event after dev injection` — open `/#agents`, inject event via `/dev/inject-federation-event`, assert event appears in panel within 5s
2. `frf-wasm publish round-trip` — call `window.__frf_dev?.wasmPublish(payload)`, assert response 200

## Quality Gates

- All Layer 1 tests pass with `SKIP_INTEGRATION=true`
- `fnm exec --using=24 pnpm typecheck` passes
- Layer 2+3 skip cleanly (no errors, just `test.skip`)
