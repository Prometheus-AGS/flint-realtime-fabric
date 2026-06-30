# Plan — phase-13-live-layer3-e2e-validation

> Authored: 2026-06-30 · Source tool: kbd-plan · Backend: openspec

---

## Ordering Rationale

The three pre-run blockers (c001–c003) must land before any live Stage 10 run.
c004 (wasm-pack in Stage 6) is independent and can proceed in parallel with
c001–c003 but must complete before the baseline commit (c005). c006 and c007
(Makefile + DEVELOPMENT.md) have no dependencies. c008 (ESLint) is independent.
All changes except c005 can be executed without a running Docker environment.

```
c001 → dev-endpoints feature flag (critical blocker)
c002 → GATEWAY_URL port fix in Stage 10
c003 → iggy service_healthy condition
c004 → wasm-pack install in Stage 6
c005 → commit .wasm-size-baseline  [G2 — manual step after DinD run]
c006 → Makefile with baseline-wasm target
c007 → docs/DEVELOPMENT.md
c008 → ESLint + env-var coercion rule in admin-ui
```

c005 is the only change that requires a live DinD run. It is documented as a
manual step and will be applied by the operator after the first successful
Stage 6 run. The remaining 7 changes are fully automatable.

---

## Changes

| # | Change ID | Title | Goal | Files | Priority |
|---|-----------|-------|------|-------|----------|
| 1 | p13-c001-dev-endpoints-feature-flag | Replace `cfg(debug_assertions)` with `dev-endpoints` Cargo feature | G1, G3 | `crates/frf-gateway/Cargo.toml`, `crates/frf-gateway/src/routes/dev.rs`, `crates/frf-gateway/src/lib.rs`, `Dockerfile`, `compose.yml` | CRITICAL |
| 2 | p13-c002-stage10-gateway-url-port-fix | Fix `GATEWAY_URL` port mismatch in Stage 10 | G1, G3 | `dagger/codegen.ts` | HIGH |
| 3 | p13-c003-iggy-service-healthy | Change iggy `depends_on` to `service_healthy` | G1, G3 | `compose.yml` | MEDIUM |
| 4 | p13-c004-stage6-wasm-pack-install | Install `wasm-pack` in Stage 6 Dagger container | G2 | `dagger/codegen.ts` | HIGH |
| 5 | p13-c005-wasm-size-baseline-commit | Commit `.wasm-size-baseline` after first Stage 6 run | G2 | `.wasm-size-baseline` | MANUAL |
| 6 | p13-c006-makefile-baseline-wasm | Add `Makefile` with `baseline-wasm` target | G4 | `Makefile` | LOW |
| 7 | p13-c007-docs-development-md | Create `docs/DEVELOPMENT.md` | G4 | `docs/DEVELOPMENT.md` | LOW |
| 8 | p13-c008-eslint-env-coercion-rule | Add ESLint + env-var coercion rule to admin-ui | G5 | `admin-ui/package.json`, `admin-ui/eslint.config.mjs` | MEDIUM |

---

## Execution Order

```
Phase: Fix pre-run blockers (run before any DinD attempt)
  1. c001 — dev-endpoints feature flag       [CRITICAL]
  2. c002 — GATEWAY_URL port fix             [HIGH]
  3. c003 — iggy service_healthy             [MEDIUM]
  4. c004 — wasm-pack in Stage 6             [HIGH]

Phase: Dev tooling (independent, no DinD required)
  5. c006 — Makefile
  6. c007 — DEVELOPMENT.md
  7. c008 — ESLint rule

Phase: Live DinD run (manual step — requires Docker host)
  8. c005 — commit .wasm-size-baseline       [MANUAL]
```

---

## Notes

- c001 is the most impactful change: without it, Stage 10 federation tests
  always fail regardless of environment.
- c002 and c003 are small targeted edits (1–2 lines each).
- c004 requires knowing which version of wasm-pack to pin; use `latest` via
  the official installer unless CI pinning is required.
- c005 is explicitly manual and cannot be automated without a DinD runner
  available in this session. The operator should run Stage 10 after c001–c004
  are merged, then commit the resulting `.wasm-size-baseline`.
- c008 introduces ESLint from scratch — the package.json, config, and a
  minimal custom rule are all new files.
