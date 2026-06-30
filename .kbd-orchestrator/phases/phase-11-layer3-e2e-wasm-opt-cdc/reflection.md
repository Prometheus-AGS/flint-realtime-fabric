# Reflection — phase-11-layer3-e2e-wasm-opt-cdc

## Goal Achievement

| Goal | Status | Notes |
|------|--------|-------|
| G1 — Layer 3 Full-Stack E2E | PARTIAL | Stage 10 env var bug (GATEWAY_URL missing) is fixed; the Stage 10 block now injects `GATEWAY_URL=http://localhost:28080` and `SKIP_INTEGRATION=false`. Actual Layer 3 test *execution* against a live compose stack was not verified in this phase (requires DinD runtime). The code path is correct. |
| G2 — wasm-opt upgrade | MET | binaryen 116 installed in Stage 6 before wasm-pack; `--no-opt` removed from wasm-pack build command. PATH shadowing ensures wasm-pack uses binaryen 116's wasm-opt instead of the bundled v105. |
| G3 — PostgreSQL CDC wiring | PARTIAL | Compose env vars wired (c001) and Postgres REPLICATION privilege + slot pre-creation added to init.sql (c002). WAL→spine fan-out runtime is not verified in this phase — that requires a running compose stack. The adapter (`frf-postgres-cdc`) is already implemented and wired. |
| G4 — CRDT Layer 2 test | MET | `admin-ui/e2e/layer2-crdt.spec.ts` created with 4 Layer 1 tests (static shape, button visible, button enabled in idle, describes apply_delta) and 2 Layer 2 tests (gated on WASM_AVAILABLE=1 + SKIP_INTEGRATION=false). Tests verify the CrdtDemoButton component and the result `data-testid="crdt-result"`. |
| G5 — Kotlin JNI test guard | MET | `tasks.withType<Test> { enabled = false }` added to `sdks/kotlin/lib/build.gradle.kts`. Compilation still runs; test execution is skipped. Clear comment explains why. |

**Overall: 3/5 MET, 2/5 PARTIAL**

## Delivered Changes

1. **p11-c001-cdc-compose-wiring** — Added 6 CDC env vars to compose.yml `gateway` service; added `postgres: { condition: service_healthy }` to `gateway.depends_on`.
2. **p11-c002-postgres-replication-setup** — Added `ALTER USER frf REPLICATION` and idempotent logical replication slot creation to `deploy/postgres/init.sql`.
3. **p11-c003-layer3-e2e-run** — Injected `GATEWAY_URL=http://localhost:28080` and `SKIP_INTEGRATION=false` into Dagger Stage 10 before the Playwright test invocation.
4. **p11-c004-crdt-layer2-test** — Created `admin-ui/e2e/layer2-crdt.spec.ts` with Layer 1 (4 tests) and Layer 2 (2 tests, gated) for the CrdtDemoButton component.
5. **p11-c005-kotlin-jni-test-guard** — Disabled Kotlin test execution in `sdks/kotlin/lib/build.gradle.kts` via `tasks.withType<Test> { enabled = false }`.
6. **p11-c006-wasm-opt-upgrade** — Removed `--no-opt`; added binaryen 116 install to Dagger Stage 6 to shadow the bundled wasm-opt v105.

## Technical Debt Introduced

- **G1 partial**: Layer 3 E2E tests have not actually been *run* against a live compose stack. The GATEWAY_URL fix is correct but unverified at runtime. A follow-up phase should run Stage 10 in an environment with Docker host access and fix any test failures.
- **G3 partial**: WAL→spine runtime fan-out is wired in compose but not smoke-tested. The `PostgresCdcConsumer` startup against the replication slot was not exercised.
- **binaryen download in CI**: The binaryen 116 tarball download from GitHub in Dagger Stage 6 adds ~30s to the WASM build stage. A cached Docker layer or pre-baked base image would eliminate this cost.
- **CRDT Layer 2 tests**: Layer 2 tests require `WASM_AVAILABLE=1` which implies a prior `wasm-pack build` run. This is gated correctly but not exercised in Stage 8 (e2e-smoke) since that stage doesn't set `WASM_AVAILABLE`. Stage 8 should set it when the WASM stage ran successfully.

## Lessons

- The `tasks.withType<Test>` guard in Kotlin was the right call: JNI loading on `System.loadLibrary` happens at class initialization, not at test call site, so no conditional skip inside a test method would help.
- The `data-testid="crdt-result"` attribute already existed on the result `<pre>` in CrdtDemoButton — spec alignment with existing attributes avoids brittle selector churn.
- Dagger Stage 10 needed both `GATEWAY_URL` and `SKIP_INTEGRATION=false` because the layer2 specs check both conditions (`!process.env["GATEWAY_URL"]` is truthy when unset, causing unconditional skip even with `SKIP_INTEGRATION` unset).

## Recommended Next Phase

**Phase 12: Layer 3 E2E Runtime Verification + Stage 8 WASM Gate + CDC Smoke**

Goals:
- G1: Run Stage 10 with DinD and fix observed Layer 3 test failures (iggy topic pre-creation, CDC consumer timing)
- G2: Set `WASM_AVAILABLE=1` in Stage 8 (e2e-smoke) so Layer 2 CRDT tests run in the standard smoke gate
- G3: Add a compose smoke test that verifies CDC consumer connects (check gateway logs for `[cdc] slot activated`)
- G4: Fix any subscribe/agent Layer 3 failures from actual Stage 10 run (requires DinD environment)
- G5: Measure WASM binary size before/after binaryen 116 optimization; add size gate (<1.5MB) to Stage 6
