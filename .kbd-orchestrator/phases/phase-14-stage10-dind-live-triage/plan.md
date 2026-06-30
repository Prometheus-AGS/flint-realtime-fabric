# Plan — phase-14-stage10-dind-live-triage

> Authored: 2026-06-30 · Source tool: kbd-plan · Backend: openspec

---

## Ordering Rationale

c001 (jwks.json) is the hardest blocker — without it the entire oathkeeper
service crashes at startup and no compose test can run. It must land first.

c002 (auth bypass env var) is the second blocker — `/v1/publish` and
`/ws/v1/subscribe` return 401 to Phase 4/5 tests. The simplest path is a
`DEV_NO_AUTH=true` gateway env var that skips JWT verification when the
`dev-endpoints` feature is active.

c003 (tenant_id in spec) and c004 (404 assertion fix) fix the Phase 6 spec
bugs. c003 is CRITICAL (400 response kills federation tests); c004 is HIGH
(test assertion mismatch causes false failure).

c005 (ENABLE_FEDERATION_STAGE gate) prevents Phase 6 Layer 3 tests from
timing out when bridges are absent. It belongs after the critical fixes but
before any live run.

c006 (wasm-size-baseline) is manual — requires a DinD Stage 6 run to
measure the binary.

c007 (Playwright retries) is deferred until flakiness data from the live run.

```
c001 → deploy/oathkeeper/jwks.json  [CRITICAL — stack won't start]
c002 → DEV_NO_AUTH gateway env + compose wiring  [HIGH — 401 on all tests]
c003 → phase6 spec tenant_id fix  [CRITICAL — 400 on federation inject]
c004 → phase6 "404 in release" assertion fix  [HIGH — false failure]
c005 → ENABLE_FEDERATION_STAGE gate  [MEDIUM — timeout prevention]
c006 → .wasm-size-baseline commit  [MANUAL — after DinD Stage 6 run]
c007 → Playwright retries evaluation  [DEFERRED — needs live flakiness data]
```

---

## Changes

| # | Change ID | Title | Goal | Files | Priority |
|---|-----------|-------|------|-------|----------|
| 1 | p14-c001-oathkeeper-jwks | Add dev HMAC JWKS to `deploy/oathkeeper/` | G1, G3 | `deploy/oathkeeper/jwks.json` | CRITICAL |
| 2 | p14-c002-gateway-dev-no-auth | Add `DEV_NO_AUTH` env var to skip JWT in `dev-endpoints` mode | G1, G3 | `crates/frf-gateway/src/routes/publish.rs`, `crates/frf-gateway/src/routes/subscribe.rs`, `crates/frf-gateway/src/config.rs`, `compose.yml` | HIGH |
| 3 | p14-c003-phase6-spec-tenant-id | Add `tenant_id` to Phase 6 spec POST bodies | G1, G3 | `admin-ui/e2e/phase6-smoke.spec.ts` | CRITICAL |
| 4 | p14-c004-phase6-spec-404-fix | Fix Phase 6 "404 in release" assertion | G1, G3 | `admin-ui/e2e/phase6-smoke.spec.ts` | HIGH |
| 5 | p14-c005-enable-federation-stage | Add `ENABLE_FEDERATION_STAGE` gate to Phase 6 Layer 3 tests | G4 | `admin-ui/e2e/phase6-smoke.spec.ts`, `dagger/codegen.ts` | MEDIUM |
| 6 | p14-c006-wasm-size-baseline | Commit `.wasm-size-baseline` after DinD Stage 6 run | G2 | `.wasm-size-baseline` | MANUAL |
| 7 | p14-c007-playwright-retries | Add `--retries=2` to Stage 10 Playwright invocation | G5 | `dagger/codegen.ts` | DEFERRED |

---

## Execution Order

```
Phase: Critical blockers (required before any DinD attempt)
  1. c001 — oathkeeper jwks.json              [CRITICAL]
  2. c002 — DEV_NO_AUTH gateway env           [HIGH]
  3. c003 — phase6 tenant_id in POST bodies   [CRITICAL]
  4. c004 — phase6 "404 in release" fix       [HIGH]

Phase: Test quality (prevents spurious timeouts)
  5. c005 — ENABLE_FEDERATION_STAGE gate      [MEDIUM]

Phase: Post-DinD run (manual / deferred)
  6. c006 — .wasm-size-baseline commit        [MANUAL]
  7. c007 — Playwright retries               [DEFERRED]
```

---

## Notes

- c001: The HMAC JWKS is a dev-only file checked into `deploy/`. It uses a
  static symmetric key — not a real secret since these credentials only grant
  access to the dev/test compose stack, never production.
- c002: The `DEV_NO_AUTH` path must be gated strictly on the `dev-endpoints`
  Cargo feature flag — if the feature is absent the env var is inert.
- c003 and c004 are both in `phase6-smoke.spec.ts` but are logically separate:
  c003 fixes the request body; c004 fixes a test assertion. Apply as one change.
- c005 adds a second skip condition to Phase 6 Layer 3 blocks:
  `process.env["ENABLE_FEDERATION_STAGE"] !== "true"`. Stage 10 leaves this
  unset (default false), skipping federation bus tests that would time out.
- c006 and c007 are explicitly deferred/manual — they require a live DinD
  environment and cannot be applied in a session without Docker.
