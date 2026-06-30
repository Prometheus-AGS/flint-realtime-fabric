# Goals — Phase 12: Layer 3 E2E Runtime Verification + Stage 8 WASM Gate + CDC Smoke

> Seeded from phase-11 reflection · 2026-06-23

## Goals

- **G1 — Layer 3 E2E runtime fix**: Run `ENABLE_INTEGRATION_STAGE=true` Dagger Stage 10 in an environment with Docker host access. Identify and fix observed Layer 3 test failures (expected: iggy topic pre-creation missing, CDC consumer startup timing, compose networking issues in DinD).

- **G2 — Stage 8 WASM gate**: Set `WASM_AVAILABLE=1` in Dagger Stage 8 (e2e-smoke) so the CRDT Layer 2 tests in `layer2-crdt.spec.ts` execute in the standard CI smoke gate, not just in the opt-in integration stage.

- **G3 — CDC smoke test**: Add a compose smoke verification that checks the CDC consumer connects to the replication slot on gateway startup. Check `docker compose logs gateway` for the `[cdc] slot activated` (or equivalent) log line within 10 seconds of startup.

- **G4 — Layer 3 test failure fixes**: After running Stage 10, fix specific failures found. Known likely failures: iggy topic pre-creation (topic must exist before publish/subscribe), agent bus WebSocket test needing an active iggy connection, federation smoke needing Tuwunel up.

- **G5 — WASM size gate**: Measure the `frf_wasm_bg.wasm` binary size produced by Stage 6 with binaryen 116 optimization active. Add a size gate assertion in Stage 6: if the optimized WASM exceeds 1.5 MB, emit a warning. Baseline the current size in `.criterion/` or a dedicated `wasm-size-baseline.json`.
