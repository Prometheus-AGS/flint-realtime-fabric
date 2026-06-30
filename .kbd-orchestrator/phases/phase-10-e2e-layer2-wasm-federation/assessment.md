# Assessment — Phase 10: Admin UI Layer 2 E2E + Federation Readiness + WASM Integration Hardening

> Written: 2026-06-21 · Tool: kbd-assess

---

## Codebase State at Phase Entry

| Artifact | Status |
|----------|--------|
| `cargo check --workspace` | PASS — zero errors |
| `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` | PASS — exits 0 |
| `cargo test -p frf-gateway` | PASS — signal_mux 3/3, signal_service, healthz |
| `cargo test -p frf-policy-cedar` | PASS — 3/3 |
| `cargo build --bin uniffi-bindgen` | PASS — exits 0 |
| `sdks/swift/Sources/FrfClient/frf.swift` | EXISTS — generated and committed |
| `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` | EXISTS — 1096 lines, committed |
| `.criterion/` baseline | EXISTS — 48 JSON files from `--save-baseline main` run |
| Dagger Stage 9 (bench) | WIRED — `ENABLE_BENCH_STAGE=true`; baseline restore included |
| Dagger Stage 10 (integration) | WIRED — `ENABLE_INTEGRATION_STAGE=true`; compose up/down + Playwright |
| `ReqwestMatrixClient` BLOCKED_ON_TUWUNEL | DOCUMENTED — doc comment committed |
| `p7-smoke.spec.ts` Layer 1 (8 tests) | PASS — UI-only, no gateway required |

---

## Gap Analysis by Goal

### G1 — Admin UI Layer 2 E2E

**Status: NOT STARTED**

Present:
- 5 E2E spec files in `admin-ui/e2e/` — all follow 3-layer pattern (UI / integration / full-stack)
- `p7-smoke.spec.ts` has Layer 1 (UI-only) and Layer 2 (gateway-required) tests already defined
- `phase4-smoke.spec.ts`, `phase5-smoke.spec.ts`, `phase6-smoke.spec.ts` follow same pattern
- `signaling.spec.ts` — simplified Layer 1 only (room join controls, CRDT button)
- `playwright.config.ts` — `testDir: "./e2e"`, `webServer: pnpm dev`, `baseURL: http://localhost:5173`

Routes in `App.tsx`:
- `/#demo/signaling` → `SignalingDemoPage` (publish + CRDT demo)
- `/#agents` → `AgentActivityPanel` (subscribe stream)
- `/` (default) → `EntitiesPage` → `EntityGraph` (entity subscribe + connection badge)

Gaps:
- No Layer 2 test for the **publish form fill** on `EntitiesPage` / `SignalingDemoPage` (form-fill POST to gateway)
- No Layer 2 test for **WebSocket subscribe stream** — `EntityGraph` shows connected/disconnected badge but no test verifies real WS connection
- No Layer 2 test for **agent session UI** — `AgentActivityPanel` Layer 2 WS tests exist in `phase5-smoke.spec.ts` but agent session start/stop not tested
- No Layer 2 test for **CRDT demo button** integration (button exists; click + result verification absent)
- `signaling.spec.ts` duplicates `p7-smoke.spec.ts` Layer 1 tests — deduplication opportunity

Work required:
1. Add `admin-ui/e2e/layer2-publish.spec.ts` — form-fill publish, expects 200/403 response logged
2. Add `admin-ui/e2e/layer2-subscribe.spec.ts` — entity stream WS connect, connected badge turns green
3. Add `admin-ui/e2e/layer2-agent.spec.ts` — agent session panel WS lifecycle
4. All Layer 2 tests: gate on `SKIP_INTEGRATION=false && GATEWAY_URL` (matching existing convention)

---

### G2 — WASM Integration Validation

**Status: PARTIAL — stub in place, real build path unverified**

Present:
- `crates/frf-wasm/` — full wasm-bindgen crate with `AgentStream`, `crdt_apply_delta` exports; wasm32-only modules gated with `#[cfg(target_arch = "wasm32")]`
- `admin-ui/vite.config.ts` — `frfWasmStubPlugin()` detects whether `frf-wasm` package is resolvable; stubs it when missing
- `admin-ui/src/types/frf-wasm.d.ts` — type stubs for `crdt_apply_delta` and `init`
- `admin-ui/src/features/agents/services/agentGrpcStream.ts` — hardcoded path `../../../../../sdks/ts/frf-wasm/frf_wasm.js`
- `sdks/ts/.gitignore` — ignores `frf-wasm/` directory
- `admin-ui/package.json` — no `frf-wasm` dependency; Vite stub resolves it at dev time

Gaps:
- `sdks/ts/frf-wasm/` directory does **not exist** (gitignored, must be built by `wasm-pack`)
- No `wasm-pack build` has been run against the updated workspace; WASM output unverified
- `admin-ui` does not declare `frf-wasm` as a pnpm workspace dep — relies on relative hardcoded path and Vite stub; fragile
- `CrdtDemoButton.tsx` imports `"frf-wasm"` via module alias (Vite stub) — different path from `agentGrpcStream.ts` hardcoded import; inconsistency
- Dagger Stage 6 (wasm-build) invokes `wasm-pack build` but no local verification has confirmed the built output is importable by admin-UI

Work required:
1. Run `wasm-pack build --target web --out-dir ../../sdks/ts/frf-wasm --out-name frf_wasm --release` from `crates/frf-wasm/`
2. Verify `sdks/ts/frf-wasm/frf_wasm.js` and `frf_wasm_bg.wasm` are produced
3. Add `"frf-wasm": "workspace:../sdks/ts/frf-wasm"` to `admin-ui/package.json` and `sdks/ts/frf-wasm/package.json` (with `"name": "frf-wasm"`)
4. Resolve import inconsistency: standardize on the `"frf-wasm"` module alias everywhere (remove hardcoded relative path in `agentGrpcStream.ts`)
5. Re-run `pnpm install` to wire the workspace dep

---

### G3 — Compose Smoke in CI

**Status: COMPOSE FILE EXISTS — untested locally**

Present:
- `compose.yml` — 7 services: `gateway`, `iggy-server`, `keto`, `oathkeeper`, `postgres`, `surrealdb` + data volumes
- `gateway` service — `healthcheck: curl /healthz`; depends on `iggy-server` and `keto`
- `iggy-server` — healthcheck on `http://localhost:3000/api/ping`
- Dagger Stage 10 polls `http://localhost:8080/healthz` up to 60s

Gaps:
- `compose.yml` references a `Dockerfile` for gateway — no `Dockerfile` confirmed at repo root
- `oathkeeper` config file not confirmed present in repo
- `keto` and `surrealdb` configs: volumes not verified against compose file
- Dagger Stage 10 installs `docker-compose-plugin` via apt — may fail on non-Debian CI images
- No local smoke run has been done to confirm all services start clean

Work required:
1. Verify `Dockerfile` exists at repo root (gateway image build)
2. Confirm oathkeeper, keto config files referenced by compose.yml are committed
3. Run `docker compose config` to validate compose file syntax
4. Run `docker compose up -d` locally and confirm all services reach healthy state
5. Fix any startup blockers (missing configs, port conflicts, missing env vars)

---

### G4 — Criterion Regression Guard

**Status: BASELINE COMMITTED — regression threshold not enforced**

Present:
- `.criterion/` — 48 JSON files (baseline `main` saved for `apply_delta` + `apply_delta_empty_base`)
- Dagger Stage 9 — runs `cargo bench -p frf-crdt --bench crdt_merge -- --baseline main`
- `--baseline main` prints comparison but does **not** fail on regression (Criterion limitation)

Gaps:
- Criterion 0.5 has no built-in `--fail-on-regression` flag
- No post-bench script parses the output and exits non-zero on >10% slowdown
- `--load-baseline` is an alternative read path but also does not enforce thresholds

Options:
- **Option A**: Add a `jq` script that parses `target/criterion/<bench>/estimates.json`, compares mean to baseline, and `exit 1` if delta > 10%
- **Option B**: Use `critcmp` (Criterion comparison CLI) with a custom threshold check
- **Recommendation**: Option A — no extra tool dependency; `jq` already needed in CI

Work required:
1. Add a threshold-check shell script `scripts/bench-regression-check.sh` that reads `target/criterion/` estimates vs baseline and fails on >10% regression
2. Wire the script into Dagger Stage 9 after the bench run

---

### G5 — Kotlin SDK Polish

**Status: PARTIAL — `frf.kt` committed; unformatted; Gradle build not verified**

Present:
- `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` — 1096 lines, UniFFI-generated, package `uniffi.frf`
- `sdks/kotlin/lib/build.gradle.kts` — `net.java.dev.jna:jna:5.14.0` dep declared; JNI lib path configured
- `sdks/kotlin/settings.gradle.kts` — root project `frf-kotlin`, includes `:lib`
- `sdks/kotlin/build.gradle.kts` — root level: `kotlin("jvm")`, `mavenCentral()`, version `0.1.0`
- `sdks/kotlin/gradle/` — Gradle wrapper present

Gaps:
- `ktlint` not installed; `frf.kt` was generated without formatting (ktlint warning at generation time)
- No Dagger stage runs `./gradlew build` in `sdks/kotlin/` to verify compilation
- `src/main/jniLibs/` directory likely empty — JNI native library (`libfrf_ffi.so` / `.dylib`) not placed
- Dagger Stage 3 runs a diff check but never validates that the `.kt` file actually compiles via Gradle

Work required:
1. Add `ktlint` formatting to Dagger Stage 3 after generation (`ktlint --format frf.kt`)
2. Add Dagger Stage 3b: copy `target/release/libfrf_ffi.so` to `sdks/kotlin/lib/src/main/jniLibs/linux-x86-64/` and run `./gradlew :lib:build`
3. Confirm the Gradle build passes with JNA dependency resolved

---

## Priority Order for Phase 10 Changes

| Priority | Goal | Rationale |
|----------|------|-----------|
| P1 | G2 — WASM integration validation | Unblocks G1 (Layer 2 CRDT tests need real WASM); blocks Dagger Stage 6 verification |
| P2 | G3 — Compose smoke | Unblocks Dagger Stage 10 (Layer 3 E2E) and G1 gateway-required tests |
| P3 | G1 — Admin UI Layer 2 E2E | Main deliverable; depends on G2 + G3 for integration tests |
| P4 | G5 — Kotlin SDK polish | Unblocks mobile consumers; independent of gateway |
| P5 | G4 — Criterion regression guard | Low risk; infrastructure improvement; independent |

---

## Open Questions for Plan

1. **`Dockerfile` at repo root**: Does it exist? The `compose.yml` gateway service references `build: context: .` with `dockerfile: Dockerfile`. If absent, compose up will fail.
2. **`frf-wasm` package name**: `sdks/ts/frf-wasm/` needs a `package.json` with `"name": "frf-wasm"` for pnpm workspace resolution. Does `wasm-pack build` generate this automatically?
3. **ktlint binary availability in Dagger**: The Kotlin CI stage uses `ghcr.io/cirruslabs/flutter:stable` — does it include `ktlint`? May need to download separately.
4. **WASM build for macOS CI**: `wasm-pack build` on the host (macOS) produces `.wasm` for wasm32, but the Dagger Stage 6 runs on `rust:1.85-slim` (Linux). The local verification in G2 should run on the host; CI stage is separate.
