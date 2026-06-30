# Goals — Phase 11: Layer 3 Full-Stack E2E + WASM Size Optimization + CDC Wiring

> Seeded from phase-10 reflection · 2026-06-22

## Goals

- **G1 — Layer 3 Full-Stack E2E**: Run `ENABLE_INTEGRATION_STAGE=true` Dagger Stage 10 (compose up + Playwright against live gateway). This is the first time Layer 3 tests run in CI; expect failures on subscribe/agent flows that need real iggy topics. Fix any failures discovered.

- **G2 — wasm-opt upgrade**: Replace the `--no-opt` workaround in Dagger Stage 6 with an explicit `wasm-opt` version ≥ 0.116 install. Measure binary size delta before and after optimization. Update `crates/frf-wasm/Cargo.toml` wasm-bindgen version if needed.

- **G3 — PostgreSQL CDC wiring**: Wire logical replication slot creation and WAL→spine fan-out for the entity subscription path in `frf-postgres-cdc`. The adapter stubs exist; this phase wires the port implementation against the `LogBroker` port.

- **G4 — CRDT Layer 2 test**: Add `layer2-crdt.spec.ts` gated on `SKIP_INTEGRATION=false`. Verify the apply-delta round-trip through the `frf-wasm` WASM module in the browser context (Playwright loading frf_wasm.js + calling `applyDelta`).

- **G5 — Kotlin JNI test guard**: Add `tasks.withType<Test> { enabled = false }` to `sdks/kotlin/lib/build.gradle.kts`. Verify that Dagger Stage 3 (`ENABLE_KOTLIN_STAGE=true`) still exits 0 with the test guard in place.
