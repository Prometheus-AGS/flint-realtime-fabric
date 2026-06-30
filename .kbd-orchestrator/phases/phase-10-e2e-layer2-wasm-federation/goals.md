# Goals — Phase 10: Admin UI Layer 2 E2E + Federation Readiness + WASM Integration Hardening

> Seeded from phase-9 reflection · 2026-06-21

## Goals

- **G1 — Admin UI Layer 2 E2E**: Implement form-fill publish flow, WebSocket subscribe stream, and agent session UI interaction tests in `admin-ui/e2e/`. Gate Layer 2 tests on `SKIP_INTEGRATION=false` so they run against the Compose stack and skip by default.

- **G2 — WASM integration validation**: Verify `frf-wasm` builds cleanly with the updated workspace; confirm `sdks/ts/frf-wasm/` output matches the admin-UI import path; ensure the WASM package is correctly consumed by the pnpm workspace.

- **G3 — Compose smoke in CI**: Run `compose.yml` locally and confirm all services (gateway, iggy, keto, oathkeeper, surrealdb, postgres) start clean and healthz polls succeed. Fix any startup issues before the full Layer 3 E2E is exercised by Dagger Stage 10.

- **G4 — Criterion regression guard**: Extend the Dagger Stage 9 bench step to fail on >10% regression. Investigate whether Criterion's `--load-baseline` or a custom threshold script is the correct mechanism.

- **G5 — Kotlin SDK polish**: Add ktlint formatting to the Dagger Kotlin bindgen stage; verify `frf.kt` compiles against `net.java.dev.jna` via `./gradlew build` in `sdks/kotlin/`.
