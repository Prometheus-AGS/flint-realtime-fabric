# Assessment — phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Status: assess_complete

---

## G1 — Layer 3 E2E Runtime Fix

**Gap: CRITICAL** — `SKIP_INTEGRATION` coercion bug causes all Layer 3 integration tests to skip even in Stage 10.

### Finding

`admin-ui/e2e/phase4-smoke.spec.ts`, `phase5-smoke.spec.ts`, and `phase6-smoke.spec.ts` all use:
```typescript
const SKIP_INTEGRATION = !!process.env["SKIP_INTEGRATION"] || !process.env["GATEWAY_URL"];
```

`!!process.env["SKIP_INTEGRATION"]` evaluates the string `"false"` as truthy (any non-empty string is truthy in JS). When Stage 10 sets `SKIP_INTEGRATION="false"`, all integration-gated tests still skip because `!!"false" === true`.

The correct check is `process.env["SKIP_INTEGRATION"] === "true"`.

**Files affected:**
- `admin-ui/e2e/phase4-smoke.spec.ts` line 19
- `admin-ui/e2e/phase5-smoke.spec.ts` line 20
- `admin-ui/e2e/phase6-smoke.spec.ts` line 21

### Additional Finding: iggy topic pre-creation

The publish route calls `state.publish_usecase.execute(req)` which internally calls `log_broker.publish(envelope)`. The `IggyBroker::publish` method requires the stream and topic to already exist — it does NOT call `ensure_channel` automatically before publishing. The compose stack starts `iggy-server` with no pre-created streams or topics.

**Risk**: First Layer 3 publish attempt will fail with "stream not found" until `ensure_channel` is called explicitly before publish. This is a known Iggy behavior (stream must be created before messages can be sent). The `PublishUseCase` does not call `ensure_channel` first.

Mitigation: The gateway startup should call `ensure_channel` for a known fixture channel, or the Phase 4 Layer 3 tests should call an `ensure_channel` endpoint before publishing.

---

## G2 — Stage 8 WASM Gate

**Gap: PRESENT** — Stage 8 (e2e-smoke) does not set `WASM_AVAILABLE=1`.

Stage 8 runs:
```typescript
const e2eSmoke = pnpmBuild
    .withExec(["pnpm", "exec", "playwright", "install", ...])
    .withWorkdir("/workspace/admin-ui")
    .withExec(["pnpm", "exec", "playwright", "test",
        "e2e/p7-smoke.spec.ts",
        "--reporter=list",
    ]);
```

It only runs `p7-smoke.spec.ts` (single file, no `WASM_AVAILABLE`). The CRDT layer2 tests (`layer2-crdt.spec.ts`) are never executed in Stage 8.

Proposed fix: Expand Stage 8 to run all `e2e/*.spec.ts` with `WASM_AVAILABLE=1` set (WASM was just built by Stage 6 and mounted via `wasmOut`). Since the admin-UI dev server is not started in Stage 8 (tests run against `/?skip-server` or static HTML), this may require spawning a `pnpm preview` or `pnpm dev` server.

**Current Stage 8 does NOT start a dev server.** It relies on Playwright `webServer` config in `playwright.config.ts`.

Check: `admin-ui/playwright.config.ts` must configure `webServer` with a start command.

---

## G3 — CDC Smoke Test

**Gap: PRESENT** — No CDC connection smoke test exists.

`compose.yml` now has `CDC_ENABLED=true` and the `depends_on: postgres: healthy`, but there is no test that verifies the CDC consumer actually establishes the replication connection. The gateway startup logs the CDC consumer status but no spec checks for it.

A CDC smoke test would:
1. `docker compose up -d` the stack
2. Poll `docker compose logs gateway` for `[cdc]` log line (or `/metrics` endpoint exposing a `cdc_slot_active` gauge)
3. Verify the slot appears in `pg_replication_slots` via `docker compose exec postgres psql -c "SELECT slot_name FROM pg_replication_slots"`

This is best implemented as a `scripts/smoke-cdc.sh` or as a new Playwright test that hits a `/debug/cdc-status` route if one exists.

**Current state:** No `/debug/cdc-status` route exists in the gateway. The simplest approach is a shell script.

---

## G4 — Layer 3 Test Failure Fixes

**Gap: BLOCKED on G1** — Cannot enumerate specific failures until the `SKIP_INTEGRATION` coercion bug is fixed and Stage 10 actually runs integration tests.

Known pre-flight failures (before running Stage 10):
1. **`SKIP_INTEGRATION` coercion bug** (see G1) — affects all 3 phase smoke specs
2. **Iggy topic pre-creation** — publish to an un-initialized stream will fail
3. **CDC layer test timing** — even with CDC wired, WAL→subscriber E2E takes >5s; Phase 4 CDC tests may need longer timeouts

---

## G5 — WASM Size Gate

**Gap: PRESENT** — No WASM binary size baseline or gate exists.

Stage 6 now builds with binaryen 116 optimization, but no size check is performed. The size of `frf_wasm_bg.wasm` with Loro CRDT compiled in is unknown — it could be 500KB or 5MB. A gate is needed to prevent silent size regressions.

No `wasm-size-baseline.json` exists. The gate would need a first-run baseline commit.

**Proposed approach:**
1. In Stage 6, after build: `wc -c sdks/ts/frf-wasm/frf_wasm_bg.wasm` and emit the size
2. Compare against a committed baseline in `.criterion/wasm-size-baseline.json`
3. Fail if > 150% of baseline (first run: just emit and commit the baseline)

---

## Change Plan Preview

Based on the assessment, 5 changes are needed in dependency order:

| # | ID | Description | Priority |
|---|-----|-------------|----------|
| 1 | p12-c001 | Fix `SKIP_INTEGRATION` coercion bug in phase4/5/6 smoke specs | CRITICAL |
| 2 | p12-c002 | Add `ensure_channel` call in gateway startup for fixture channel | HIGH |
| 3 | p12-c003 | Expand Stage 8 to run all e2e specs with `WASM_AVAILABLE=1` | MEDIUM |
| 4 | p12-c004 | CDC smoke shell script (`scripts/smoke-cdc.sh`) | MEDIUM |
| 5 | p12-c005 | WASM size baseline + size gate in Stage 6 | LOW |

c001 must land before c002 for Layer 3 tests to be meaningful. c003/c004/c005 are independent.
