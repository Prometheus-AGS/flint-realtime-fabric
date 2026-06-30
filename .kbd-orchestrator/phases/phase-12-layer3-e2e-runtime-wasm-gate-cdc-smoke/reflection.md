# Reflection — phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

> Authored: 2026-06-30 · Source tool: kbd-reflect

---

## Goal Achievement

| Goal | Status | Notes |
|------|--------|-------|
| G1 — Layer 3 E2E runtime fix (`SKIP_INTEGRATION` coercion + iggy pre-creation) | **MET** | All three phase smoke specs corrected to `=== "true"` pattern; gateway now calls `ensure_channel` at startup |
| G2 — Stage 8 WASM gate | **MET** | `WASM_AVAILABLE=1` set + `e2e/` directory run in Stage 8; CRDT Layer 2 tests now execute in standard CI smoke |
| G3 — CDC smoke test | **MET** | `scripts/smoke-cdc.sh` created; polls `pg_replication_slots` for `active=true` up to 30s with enhanced diagnostics |
| G4 — Layer 3 test failure fixes | **PARTIAL** | Pre-flight blockers (coercion bug, iggy pre-creation) are fixed. Actual Stage 10 runtime failures cannot be confirmed without a live DinD environment; that live validation is deferred to Phase 13 |
| G5 — WASM size gate | **MET** | Stage 6 emits binary size and compares against `.wasm-size-baseline` when present; `.wasm-size-baseline.example` documents the baselining workflow; baseline absent pending first live run |

**Overall: 4/5 MET, 1/5 PARTIAL — execute phase considered complete.**

---

## Delivered Changes

| Change | Files | Outcome |
|--------|-------|---------|
| p12-c001-skip-integration-coercion-fix | `admin-ui/e2e/phase{4,5,6}-smoke.spec.ts` | DONE |
| p12-c002-ensure-channel-gateway-startup | `crates/frf-gateway/src/main.rs` | DONE |
| p12-c003-stage8-wasm-gate | `dagger/codegen.ts` | DONE |
| p12-c004-cdc-smoke-script | `scripts/smoke-cdc.sh` | DONE |
| p12-c005-wasm-size-gate | `dagger/codegen.ts`, `.wasm-size-baseline.example` | DONE |

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA | 0/5 (no artifact-refiner run — changes < 3 files each or pre-applied) |
| First-pass pass rate | N/A |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

QA gate was skipped per the per-change QA rule: all changes modified fewer than 3 files, and c001–c004 were pre-applied from prior session work. No constraint violations detected on manual review.

---

## Technical Debt Introduced

1. **`.wasm-size-baseline` absent** — the size gate is wired but inert until the first successful Stage 6 run produces a baseline to commit. This is intentional (first-run bootstrap) but must not be forgotten. Whoever runs Stage 6 next should commit the baseline file immediately.

2. **G4 validation deferred** — the coercion fix and iggy pre-creation fixes are in code, but Stage 10 has not been exercised in a live DinD environment. The actual set of Layer 3 test failures (CDC timing, federation Tuwunel dependency, agent bus WebSocket test requirements) remains unknown. Phase 13 must include a live Stage 10 run as its first task.

3. **Fixture channel UUID is a compile-time constant** — `00000000-0000-0000-0000-000000000001` is hardcoded in `main.rs`. This is acceptable for a development fixture but should be sourced from `GatewayConfig` (a new `fixture_tenant_id` env var, defaulting to this value) in a future hardening pass.

---

## Lessons Captured

1. **JS boolean coercion trap with env vars** — `!!process.env["X"]` is wrong when `X` may be set to the string `"false"`. The canonical pattern in this codebase is `process.env["X"] === "true"`. This pattern should be validated by a lint rule (`no-boolean-coercion-on-env`) or a pre-commit check. Consider adding an ESLint custom rule to the admin-UI workspace.

2. **Iggy requires explicit stream+topic creation** — `IggyBroker::publish` does not auto-create the stream/topic. This is a foot-gun for any new Layer 3 test that publishes to a previously unseen channel. Document this in `crates/frf-broker-iggy/README.md` or as a `# INVARIANT:` comment at the `IggyBroker::publish` signature.

3. **CDC smoke is more useful than a Playwright test** — using `pg_replication_slots` directly (via `docker compose exec psql`) gives a definitive binary answer about whether the CDC slot is active, without depending on log parsing or gateway HTTP routes. Prefer this pattern for future infrastructure smoke tests.

4. **WASM size baselining needs a workflow, not just a file** — the size gate is only as useful as the discipline around updating the baseline. The `.wasm-size-baseline.example` documents this, but a Makefile target (`make baseline-wasm`) would make it frictionless and reduce the chance the baseline gets stale.

---

## Recommended Next Phase Focus

Phase 13 (if created) should focus on **live Layer 3 E2E validation**:

1. **First task**: run `ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts` in a DinD-capable environment and capture the actual Stage 10 output. Triage all failures by category (iggy, CDC timing, federation, auth).
2. **Commit `.wasm-size-baseline`** from the first successful Stage 6 run.
3. **Fix any Stage 10 failures** found: likely categories based on assessment are iggy startup race, CDC timing (need longer poll), Tuwunel dependency (may need to skip or stub federation tests in Stage 10).
4. **Add `make baseline-wasm` target** and a note to `docs/DEVELOPMENT.md`.
5. **ESLint rule for env var coercion** in `admin-ui/` to catch future `!!process.env["X"]` uses.

If the project roadmap is complete at Phase 12, the next action is a **production readiness pass**: security audit of the compose stack, secret rotation procedure, and tagging a `v0.1.0` release.
