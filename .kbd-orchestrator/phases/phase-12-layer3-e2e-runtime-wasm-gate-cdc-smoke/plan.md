# Plan — phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Change Count: 5

## Ordering Rationale

**Chain A (must sequence — Layer 3 correctness):**
1. `p12-c001` — Fix `SKIP_INTEGRATION` coercion bug (prerequisite for any Layer 3 integration tests to run)
2. `p12-c002` — Ensure fixture channel at gateway startup (prerequisite for first Layer 3 publish to succeed)

c001 and c002 are independent of each other but both must land before Layer 3 tests can execute meaningfully. Execute in parallel.

**Chain B (independent — execute after or in parallel with Chain A):**
3. `p12-c003` — Stage 8 WASM gate (Dagger codegen change only)
4. `p12-c004` — CDC smoke script (new shell script, no deps)
5. `p12-c005` — WASM size gate (Dagger codegen change + baseline file)

c003 and c005 both touch `dagger/codegen.ts` — execute sequentially (c003 first, c005 second) to avoid merge conflicts.

## Ordered Change List

| Order | Change ID | Description | Depends On |
|-------|-----------|-------------|------------|
| 1 | p12-c001-skip-integration-coercion-fix | Fix `!!env` → `=== "true"` in 3 smoke specs | — |
| 2 | p12-c002-ensure-channel-gateway-startup | Call ensure_channel for fixture channel at startup | — |
| 3 | p12-c003-stage8-wasm-gate | Stage 8: run all e2e/ with WASM_AVAILABLE=1 | — |
| 4 | p12-c004-cdc-smoke-script | scripts/smoke-cdc.sh | — |
| 5 | p12-c005-wasm-size-gate | Stage 6: size gate + .wasm-size-baseline | c003 (codegen.ts) |

## Recommended Execution Order

- c001 + c002 + c004 in parallel (all touch different files)
- c003 after c001/c002 (logical sequence, can overlap with c004)
- c005 after c003 (same file: dagger/codegen.ts)

## Files Touched Summary

| Change | Files Modified/Created |
|--------|------------------------|
| c001 | `admin-ui/e2e/phase4-smoke.spec.ts`, `phase5-smoke.spec.ts`, `phase6-smoke.spec.ts` |
| c002 | `crates/frf-gateway/src/main.rs` |
| c003 | `dagger/codegen.ts` |
| c004 | `scripts/smoke-cdc.sh` (NEW) |
| c005 | `dagger/codegen.ts`, `.wasm-size-baseline` (NEW) |

## Exit Criteria (Phase Level)

All 5 changes complete AND:
- `grep -r "SKIP_INTEGRATION" admin-ui/e2e/phase{4,5,6}-smoke.spec.ts` shows `=== "true"` pattern
- Gateway startup log shows fixture channel pre-creation attempt (non-fatal warning acceptable)
- Dagger Stage 8 runs `e2e/` directory with `WASM_AVAILABLE=1`
- `scripts/smoke-cdc.sh` is executable and exits 0 against a live compose stack
- Stage 6 emits WASM binary size and baseline comparison to stdout
