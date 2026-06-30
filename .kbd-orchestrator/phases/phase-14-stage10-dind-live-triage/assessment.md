# Assessment — phase-14-stage10-dind-live-triage

> Authored: 2026-06-30 · Source tool: kbd-assess

---

## Scope

Assess the current codebase against the 5 Phase 14 goals:
- G1: Stage 10 live DinD run + failure triage
- G2: Commit `.wasm-size-baseline`
- G3: Fix Stage 10 runtime failures
- G4: ENABLE_FEDERATION_STAGE gate for Tuwunel-absent CI
- G5: Playwright `--retries=2` / sharding evaluation

---

## Codebase Inspection

### Stage 10 post-Phase-13 state

Phase 13 resolved three structural pre-run blockers. Stage 10 (`dagger/codegen.ts`
lines 288–322) now:

- Polls `http://localhost:8080/healthz` (30 × 2s = 60s) — **port is correct**
- Sets `GATEWAY_URL=http://localhost:8080` — **correct for DinD**
- Sets `SKIP_INTEGRATION=false` — **correct** (`=== "true"` pattern throughout)
- Composes with `iggy` `service_healthy` — **race fixed**
- Builds gateway with `CARGO_FEATURES=dev-endpoints` — **endpoint accessible**

These fixes are verified and in place.

---

## Critical New Findings

### Finding 1 — CRITICAL: `deploy/oathkeeper/jwks.json` absent

`deploy/oathkeeper/config.yml` configures the JWT authenticator with:

```yaml
jwks_urls:
  - file:///etc/config/oathkeeper/jwks.json
```

`deploy/oathkeeper/` contains only `config.yml` and `rules.json`. **No `jwks.json`
exists.** Oathkeeper will fail to start because it cannot load the JWKS file on
startup, making the entire compose stack non-functional.

**Impact:** `docker compose up -d` will start oathkeeper, but the oathkeeper
process will crash or refuse to authenticate any JWT, causing all proxy traffic
to fail (except `/healthz` which uses the `anonymous` authenticator).

**Fix required:** Provide a `deploy/oathkeeper/jwks.json` with a valid HMAC or
RSA key set — even a dev-only JWKS with a symmetric key is sufficient for
Stage 10 integration tests.

### Finding 2 — CRITICAL: Phase 6 spec sends wrong fields to `/dev/inject-federation-event`

The Rust handler `InjectFederationEventRequest` expects:

```rust
pub struct InjectFederationEventRequest {
    pub tenant_id: String,   // required — must be valid UUID
    pub protocol: String,
    pub source: String,
    pub content: serde_json::Value,
}
```

The Phase 6 smoke spec sends (lines 149–156, 171–178):

```typescript
{
    protocol: "matrix",
    source: "!smoke-room:matrix.org",
    content: { type: "text_delta", delta: `matrix smoke ${runId}` },
    // NOTE: no tenant_id field
}
```

`tenant_id` is absent — serde will fail to deserialise, returning HTTP 422
(Unprocessable Entity) or the handler's explicit 400 from the UUID parse.

**Impact:** Every Phase 6 Layer 3 federation test will fail with a 4xx error
immediately after POSTing to `/dev/inject-federation-event`.

**Fix:** Either (a) add `tenant_id: "00000000-0000-0000-0000-000000000002"` to
each spec POST body, or (b) make `tenant_id` optional in the Rust struct
(`Option<String>`) and supply a default.

### Finding 3 — HIGH: Phase 4/5 E2E tests POST to `/v1/publish` without `Authorization` header

`publish.rs` line 55:

```rust
let Some(token) = bearer_token(&headers) else {
    return StatusCode::UNAUTHORIZED.into_response();
};
```

Phase 4 CDC tests (`phase4-smoke.spec.ts` lines 113–123, 141–151) POST to
`GATEWAY_URL/v1/publish` with only `Content-Type: application/json` — no
`Authorization: Bearer` header. These tests will receive `401 Unauthorized`.

Similarly, `phase5-smoke.spec.ts` (lines 109, 133) POSTs to
`GATEWAY_URL/v1/publish` without auth.

**Fix options:**
- (a) Add a dev-mode bypass: make the gateway accept requests without auth
  when `dev-endpoints` feature is active and a special header is present, OR
- (b) Generate a static dev JWT signed with the JWKS key and include it in
  Stage 10 env vars, OR
- (c) Add a `no-auth-required` env var to the gateway that disables JWT
  verification in compose environments (simplest for Stage 10).

### Finding 4 — HIGH: Phase 6 test `test.skip(SKIP_INTEGRATION, …)` has inverted semantics for the "404 in release" assertion

Phase 6 Layer 1 test (line 97–110):

```typescript
test("dev injection endpoint returns 404 in release builds (or is absent)", async ({
  request,
}) => {
  test.skip(SKIP_INTEGRATION, "set GATEWAY_URL to verify dev endpoint absence in release");
  // Expects 404...
  expect(resp.status()).toBe(404);
});
```

This test **only runs when `SKIP_INTEGRATION=false`** (i.e., when Stage 10 is
running). But in Stage 10, the compose gateway is built **with
`dev-endpoints` feature**, so the endpoint returns `202 Accepted`, not `404`.
The test will fail with `expected 404, got 202`.

The test was written to verify that production builds omit the endpoint. But
Stage 10 deliberately enables `dev-endpoints`, so this assertion is incorrect
for Stage 10.

**Fix:** Change the expected status from `404` to one of `[202, 404]`, or skip
this specific test when `CARGO_FEATURES` includes `dev-endpoints` by gating it
on a separate env var (`DEV_ENDPOINTS_ENABLED`), or delete the test (the
feature flag is the enforcement mechanism — the test is redundant).

### Finding 5 — MEDIUM: `tenant_id` UUID format used in Phase 6 Layer 1 synthetic store injection is not a valid v4 UUID

Phase 6 Layer 1 tests (lines 54–62) inject a synthetic event into the store
via `window.__frf_dev.agentEventStore` using:

```typescript
tenant_id: "00000000-0000-0000-0000-000000000002",
```

This is a nil-UUID (version 0). This is fine for the UI layer test since it
bypasses the gateway entirely. No fix needed for Stage 10; recorded for
awareness.

### Finding 6 — MEDIUM: No `ENABLE_FEDERATION_STAGE` gate in any spec or pipeline

G4 goal asks for an `ENABLE_FEDERATION_STAGE` env var to gate Phase 6 Layer 3
federation tests (which require Matrix/ATProto bridges running). Currently
compose does not set `MATRIX_HOMESERVER_URL` or `ATPROTO_JETSTREAM_URL`, so the
bridges do not start. The Phase 6 Layer 3 tests still run (gated only on
`SKIP_INTEGRATION=false`) and will likely time out waiting for events that
never arrive on the WebSocket.

**Fix:** Add `ENABLE_FEDERATION_STAGE` env var check to the Phase 6 Layer 3
describe blocks, and wire it in Stage 10.

### Finding 7 — LOW: No `--retries` or `--shard` in Stage 10 Playwright invocation

Stage 10 runs:

```typescript
.withExec(["pnpm", "exec", "playwright", "test", "e2e/", "--reporter=list"])
```

No `--retries`, `--max-failures`, or sharding configured. This is expected for
a first live run — the value of adding retries should be determined after
observing flakiness patterns from the live run.

---

## Per-Goal Gap Analysis

### G1 — Run Stage 10 live in DinD; triage failures

**Status: READY TO RUN (with blocking issues that will cause failures)**

The pre-run structural blockers from Phase 13 are fixed. However, three
runtime failures are now predictable before the first live run:

| Failure | Root Cause | Severity |
|---------|-----------|----------|
| oathkeeper crashes on startup | `jwks.json` absent in `deploy/oathkeeper/` | CRITICAL |
| Phase 6 Layer 3 federation tests → 400 | `tenant_id` missing in spec POST bodies | CRITICAL |
| Phase 4/5 publish tests → 401 | No auth token in spec POST bodies | HIGH |
| Phase 6 Layer 1 "404 in release" test → `expected 404 got 202` | Feature flag enabled in compose | HIGH |
| Phase 6 Layer 3 federation tests time out | No Matrix/ATProto bridges in compose | MEDIUM (expected) |

Recommend fixing issues 1–4 before the DinD live run to get a clean signal.

### G2 — Commit `.wasm-size-baseline`

**Status: NOT STARTED (pending first Stage 6 run in DinD)**

`make baseline-wasm` procedure exists. No additional code changes needed.

### G3 — Fix Stage 10 runtime failures

**Status: 4 fixes identified (see above findings 1–4)**

All are automatable code changes:

1. `deploy/oathkeeper/jwks.json` — new file with dev HMAC JWKS
2. `admin-ui/e2e/phase6-smoke.spec.ts` — add `tenant_id` to POST bodies
3. Gateway or Stage 10 — disable auth in compose mode (or add demo JWT to tests)
4. `admin-ui/e2e/phase6-smoke.spec.ts` — fix "404 in release" assertion

### G4 — `ENABLE_FEDERATION_STAGE` gate

**Status: NOT STARTED**

Need to:
- Add `ENABLE_FEDERATION_STAGE` check in Phase 6 Layer 3 describe block
- Wire env var in Stage 10 (default `false`)

### G5 — Playwright `--retries=2` evaluation

**Status: DEFERRED until G1 live run**

No code changes until flakiness patterns are observed from a live Stage 10 run.

---

## Auth Strategy Recommendation

The simplest fix for Finding 3 (401 on publish) for Stage 10 is to add a
`SKIP_AUTH` / `DEV_NO_AUTH` env var to the gateway that bypasses JWT
verification in the `publish` and `subscribe` routes when the `dev-endpoints`
feature is active. This avoids needing to mint a valid JWT in the test
environment while keeping auth enforcement in production-equivalent builds.

Alternative: generate a static dev JWKS + JWT, bake it into compose env, and
pass it in test request headers. This tests the full auth path but requires
significantly more setup.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| DinD unavailable in session | HIGH | Blocks G1, G2 | All code fixes in this phase don't require DinD; operator runs Stage 10 after changes land |
| oathkeeper with in-memory JWKS resets on restart | MEDIUM | Ephemeral JWKS in compose acceptable | Stage 10 is single-run; persistence not needed |
| Static JWKS in deploy/ checked into git | LOW | Not a real secret — dev only, no prod path | Document as dev-only in DEVELOPMENT.md |
| Phase 6 Layer 3 tests still flaky after bridges absent | HIGH | Expected failure — needs `ENABLE_FEDERATION_STAGE` gate | G4 addresses this |

---

## Summary

| Goal | Status | Blockers |
|------|--------|---------|
| G1 — Stage 10 live run | READY (with known failures) | 4 runtime failures are predictable; fix before live run |
| G2 — `.wasm-size-baseline` commit | NOT STARTED | Awaits DinD Stage 6 run |
| G3 — Fix Stage 10 runtime failures | 4 IDENTIFIED | jwks.json absent, missing tenant_id, 401 on publish, 404 test inversion |
| G4 — `ENABLE_FEDERATION_STAGE` gate | NOT STARTED | New env var + spec guard needed |
| G5 — Playwright retries/sharding | DEFERRED | Await live run flakiness data |

**Recommended plan order:**

1. `deploy/oathkeeper/jwks.json` — dev HMAC JWKS (CRITICAL — blocks stack startup)
2. Gateway `DEV_NO_AUTH` bypass or demo JWT in Stage 10 env (HIGH — 401 on all publish tests)
3. Phase 6 spec `tenant_id` in POST bodies (CRITICAL — 400 on federation inject)
4. Phase 6 "404 in release" test fix (HIGH — test correctness)
5. Phase 6 Layer 3 `ENABLE_FEDERATION_STAGE` gate (MEDIUM — prevents spurious timeout failures)
6. `.wasm-size-baseline` commit procedure (manual — after DinD run)
