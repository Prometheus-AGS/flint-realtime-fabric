# Plan — Phase 10: Admin UI Layer 2 E2E + WASM Integration + Federation Readiness

> Written: 2026-06-21 · Tool: kbd-plan

---

## Change Order

| # | Change ID | Goal | Rationale |
|---|-----------|------|-----------|
| 1 | `p10-c001-wasm-workspace-wiring` | G2 | Build WASM, wire pnpm dep, fix import — unblocks c003 and c006 |
| 2 | `p10-c002-compose-smoke` | G3 | Validate compose stack starts — unblocks Layer 2 E2E gateway-required tests |
| 3 | `p10-c003-layer2-e2e-specs` | G1 | Add 3 Layer 2 spec files — main deliverable of this phase |
| 4 | `p10-c004-kotlin-gradle-build` | G5 | ktlint format + Gradle build — independent, no gateway needed |
| 5 | `p10-c005-criterion-regression-guard` | G4 | Regression guard script — independent infra improvement |
| 6 | `p10-c006-wasm-dagger-verify` | G2 | Dagger Stage 6 output verification — depends on c001 confirming output shape |

---

## Change Summaries

### p10-c001 — WASM Workspace Wiring
- Run `wasm-pack build --target web --out-dir ../../sdks/ts/frf-wasm --out-name frf_wasm --release` from `crates/frf-wasm/`
- Add `"frf-wasm": "workspace:../sdks/ts/frf-wasm"` to `admin-ui/package.json`
- Fix `agentGrpcStream.ts` import from hardcoded relative path to `"frf-wasm"`
- Run `pnpm install` to wire workspace dep
- Verify typecheck passes

### p10-c002 — Compose Smoke
- Check `Dockerfile` exists at repo root; create minimal multi-stage gateway image if missing
- Verify `deploy/oathkeeper/config.yml` and `deploy/keto/keto.yml` present
- Run `docker compose config` to validate syntax
- Run `docker compose up -d` and poll `/healthz` for 60s
- Run `docker compose down`

### p10-c003 — Layer 2 E2E Specs
- Create `admin-ui/e2e/layer2-publish.spec.ts` (publish form-fill on `/#demo/signaling`)
- Create `admin-ui/e2e/layer2-subscribe.spec.ts` (WS connect badge on `/`)
- Create `admin-ui/e2e/layer2-agent.spec.ts` (agent session start/stop on `/#agents`)
- All tests gate on `SKIP_INTEGRATION !== 'true'`

### p10-c004 — Kotlin Gradle Build
- Install ktlint, run `ktlint --format` on `frf.kt`
- Add `.gitignore` entry for JNI native libs in `sdks/kotlin/`
- Run `./gradlew :lib:compileKotlin` from `sdks/kotlin/`
- Confirm zero compilation errors

### p10-c005 — Criterion Regression Guard
- Create `scripts/bench-regression-check.sh` — compares `target/criterion/*/new/estimates.json` against `.criterion/*/main/estimates.json`, fails on >10% regression
- Make script executable (`chmod +x`)
- Add script invocation to Dagger Stage 9 after bench run
- Verify script passes locally against committed baseline

### p10-c006 — WASM Dagger Stage 6 Verification
- Add post-build `sh -c` check to Dagger Stage 6 for `frf_wasm.js` and `frf_wasm_bg.wasm`
- Add `jq` check that `package.json` has `"name": "frf-wasm"`
- Typecheck `dagger/codegen.ts` after edit

---

## Dependencies

```
c001 ──► c006 (c006 verifies the Dagger path for what c001 confirms locally)
c002 ──► c003 (c003 Layer 2 gateway tests need compose stack working)
c001 ──► c003 (c003 CRDT Layer 2 test needs real frf-wasm package)
c004 is independent
c005 is independent
```

---

## Exit Criteria for Phase 10

- [ ] `wasm-pack build` produces `frf_wasm.js` + `frf_wasm_bg.wasm`
- [ ] `pnpm install` resolves `frf-wasm` as workspace dep
- [ ] `pnpm typecheck` passes (no `any` type issues from new WASM import)
- [ ] `docker compose up -d` starts all 7 services; gateway `/healthz` 200
- [ ] 3 new Layer 2 spec files exist and pass with gateway running
- [ ] `SKIP_INTEGRATION=true pnpm e2e` still exits 0 (Layer 2 tests skipped)
- [ ] `./gradlew :lib:compileKotlin` exits 0 from `sdks/kotlin/`
- [ ] `scripts/bench-regression-check.sh` exits 0 against committed baseline
- [ ] Dagger Stage 6 output verification check is in `codegen.ts`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes
