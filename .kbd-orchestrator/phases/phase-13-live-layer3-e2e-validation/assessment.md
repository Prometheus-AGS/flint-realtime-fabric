# Assessment — phase-13-live-layer3-e2e-validation

> Authored: 2026-06-30 · Source tool: kbd-assess

---

## Scope

Assess the current codebase against the 5 goals defined for Phase 13:
live Layer 3 E2E validation, WASM baseline commit, Stage 10 failure triage and
fixes, `make baseline-wasm` target, and ESLint env-var coercion rule.

---

## Codebase Inspection Summary

### Stage 10 (`ENABLE_INTEGRATION_STAGE=true`)

Stage 10 in `dagger/codegen.ts` (lines 284–316) does the following inside a
DinD container:

1. `docker compose up -d` — starts the full stack (gateway, iggy, keto,
   oathkeeper, surrealdb, postgres).
2. Polls `http://localhost:8080/healthz` (max 60s / 30 iterations × 2s) for
   gateway readiness.
3. Sets `GATEWAY_URL=http://localhost:28080`, `SKIP_INTEGRATION=false`,
   `WASM_AVAILABLE=1`.
4. Runs `pnpm exec playwright test e2e/ --reporter=list` in `admin-ui/`.
5. `docker compose down`.

**Findings:**

| Finding | Severity | Detail |
|---------|----------|--------|
| Port mismatch: healthz polls `:8080`, GATEWAY_URL is `28080` | HIGH | Inside the Dagger container network, the gateway binds on `:8080`. `compose.yml` maps `28080:8080` host→container, but `GATEWAY_URL` is set to `http://localhost:28080`. Inside DinD the gateway container port is `:8080` (internal); `localhost:28080` is the host-side mapping. This means E2E tests POST to `:28080` which may be unreachable inside the Dagger runner. The `GATEWAY_URL` must match the actual reachable address: `http://gateway:8080` (service name in compose network) or `http://localhost:8080` (if bridged). |
| `/dev/inject-federation-event` absent in release build | CRITICAL | `dev.rs` gates the endpoint with `#[cfg(debug_assertions)]`. The Dockerfile builds with `--release`, so `debug_assertions` is disabled. Phase 6 federation smoke tests POST to this endpoint; they will receive 404. Fix options: (a) use a Cargo feature flag (`dev-endpoints`) instead of `cfg(debug_assertions)`, or (b) use a separate `compose.override.yml` that builds a debug binary for CI. |
| Iggy startup race | MEDIUM | `compose.yml` has a healthcheck for iggy (`iggy me` TCP check), but the gateway's `depends_on.iggy-server.condition` is `service_started`, not `service_healthy`. If iggy is still initialising when the gateway connects, publish will fail. Need `condition: service_healthy` for iggy in `compose.yml`. |
| `ENABLE_INTEGRATION_STAGE` env coercion is correct | OK | Uses `=== "true"` pattern (line 284). No fix needed. |
| `SKIP_INTEGRATION=false` in Stage 10 is correct | OK | Enables all gated tests. |

### WASM size baseline (G2)

- `.wasm-size-baseline` is **absent** — Stage 6 runs in "no-baseline / first-run" mode.
- `.wasm-size-baseline.example` exists documenting the baselining workflow.
- Stage 6 WASM build (`wasm-pack build crates/frf-wasm`) requires a DinD
  environment with `wasm-pack` installed in the Dagger container.
- No live run of Stage 6 has been executed; the binary size is unknown.

**Gap:** Cannot commit a baseline until Stage 6 runs end-to-end in a DinD
environment and produces `frf_wasm_bg.wasm`.

### `make baseline-wasm` / `Makefile` (G4)

- No `Makefile`, `GNUmakefile`, or `justfile` exists in the workspace root.
- `docs/DEVELOPMENT.md` does not exist (only `IMPLEMENTATION-PLAN.md`,
  `PROMETHEUS-BASE-RULES.md`, and `decisions/`).

**Gap:** Both artifacts must be created from scratch.

### ESLint / env-var coercion rule (G5)

- `admin-ui/package.json` has no `eslint` or `eslint-plugin-*` devDependencies.
- No `eslint.config.*` or `.eslintrc.*` file exists under `admin-ui/`.
- The `package.json` `scripts` block has no `lint` script.
- All three `phase{4,5,6}-smoke.spec.ts` files correctly use `=== "true"`.
- `layer2-crdt.spec.ts` uses `=== "true"` for `SKIP_INTEGRATION` ✓.
- `layer2-publish.spec.ts`, `layer2-subscribe.spec.ts`, `layer2-agent.spec.ts`
  all use `=== "true"` ✓.

**Gap:** ESLint is entirely absent. Must add ESLint + a custom rule (or
`eslint-plugin-unicorn` / `eslint-plugin-n`) that flags `!!process.env["X"]`
and requires `process.env["X"] === "true"`.

---

## Per-Goal Gap Analysis

### G1 — Run Stage 10 live in DinD; triage failures

**Status: BLOCKED by two pre-run bugs that must be fixed first:**

1. **CRITICAL — release build omits `/dev/inject-federation-event`**: Phase 6
   federation smoke tests will 404 in Stage 10 unless the endpoint is enabled
   in the image used by compose.
2. **HIGH — `GATEWAY_URL` port mismatch**: E2E tests POST to `:28080` which is
   the host-side mapped port; inside DinD the gateway is reachable at `:8080`
   via `localhost` or via the Docker compose service name `gateway`. Need to set
   `GATEWAY_URL=http://localhost:8080` (or `http://gateway:8080`) in Stage 10.

Fix these before attempting a live Stage 10 run; otherwise the run will produce
misleading failures that don't reflect real application behaviour.

**Additional pre-run concern:**

3. **MEDIUM — iggy `depends_on` should be `service_healthy`**: prevents the
   gateway from connecting before iggy finishes init.

### G2 — Commit `.wasm-size-baseline`

**Status: NOT STARTED — blocked on a successful Stage 6 run in DinD.**

Stage 6 requires `wasm-pack` installed in the Dagger runner image. The current
Dagger Node 24 base image (`node:24-slim`) does not include `wasm-pack` or
`wasm-opt`. The WASM stage must install them via `apt-get` or `cargo install
wasm-pack`.

Confirm Stage 6 actually completes in the Dagger pipeline. After the first
successful run, `wc -c sdks/ts/frf-wasm/frf_wasm_bg.wasm > .wasm-size-baseline`
and commit.

### G3 — Fix Stage 10 failures

**Status: NOT STARTED — depends on G1 live run for full triage. Known pre-run
blockers identified:**

| Fix | File | Action |
|-----|------|--------|
| Release build omits dev endpoint | `crates/frf-gateway/src/routes/dev.rs`, `Cargo.toml` | Replace `#[cfg(debug_assertions)]` with `#[cfg(feature = "dev-endpoints")]` and add the feature to `frf-gateway/Cargo.toml`; enable in `compose.yml` via `CARGO_FLAGS` env build arg |
| `GATEWAY_URL` port mismatch | `dagger/codegen.ts` line 303 | Change `.withEnvVariable("GATEWAY_URL", "http://localhost:28080")` → `http://localhost:8080` (ports match inside DinD container network) |
| Iggy `service_started` race | `compose.yml` | Change `iggy-server` `depends_on` condition from `service_started` → `service_healthy` in the gateway service block |

### G4 — `make baseline-wasm` target + `docs/DEVELOPMENT.md`

**Status: NOT STARTED — both files must be created.**

Minimal `Makefile` with `baseline-wasm` target:
```makefile
.PHONY: baseline-wasm
baseline-wasm:
	ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts 2>&1 | \
	  grep -oP '(?<=WASM binary size: )\d+' | tail -1 > .wasm-size-baseline
	git add .wasm-size-baseline
	git commit -m "chore: update WASM binary size baseline"
```

`docs/DEVELOPMENT.md` must document: WASM baselining workflow, Stage 10 DinD
requirements, CDC slot verification, local compose stack startup.

### G5 — ESLint + env-var coercion rule

**Status: NOT STARTED — ESLint entirely absent from admin-ui.**

Required changes:
1. Add `eslint`, `@typescript-eslint/eslint-plugin`, `@typescript-eslint/parser`
   to `admin-ui/package.json` devDependencies.
2. Add `lint` script: `"lint": "eslint e2e/ src/ --ext .ts,.tsx"`.
3. Create `admin-ui/eslint.config.mjs` with TypeScript config and a custom rule
   (or `eslint-plugin-unicorn` `prefer-boolean-as-predicate` equivalent) that
   flags `!!process.env`.
4. Wire lint into Dagger Stage 7 (after `pnpm install`, before `pnpm build`).

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| DinD not available in dev environment | HIGH | Blocks G1, G2 | Document how to run with `--privileged` docker; consider a remote CI-only path for Stage 10 |
| `wasm-pack` missing in Dagger node image | HIGH | Blocks G2 | Add `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf \| sh` to Stage 6 container setup, or use a custom base image |
| iggy startup takes >60s in DinD | MEDIUM | Stage 10 flaky | Increase healthz poll retries from 30×2s to 60×2s (120s total); or wait on iggy compose healthcheck before starting the gateway |
| Federation E2E blocked by release build | HIGH | Fails all phase6 smoke tests | Use `dev-endpoints` Cargo feature for compose builds |
| ESLint rule too broad | LOW | False positives | Scope the rule to the exact `!!process.env` pattern |

---

## Open Questions for Plan

1. **DinD environment**: Does the team have a CI runner (GitHub Actions, GitLab, etc.) with `--privileged` or `docker:dind` available? Stage 10 cannot run without it. If not, should Stage 10 be documented as "manual-only" for now?

2. **`dev-endpoints` Cargo feature vs. separate build profile**: Using a Cargo feature is cleaner (one binary, no extra Dockerfile), but the compose build needs a build arg. Using a `docker compose.ci.yml` override is simpler for now. Which approach?

3. **Stage 6 WASM installer**: Should `wasm-pack` be baked into a custom Dagger base image or installed per-run via shell? Per-run is simpler; custom image is faster.

4. **Makefile vs. justfile**: The project has no `just` dependency documented. `Makefile` is universally available. Proceed with `Makefile`?

5. **`docs/DEVELOPMENT.md` scope**: Should it be a quick-start only (local compose, CDC smoke, WASM baseline), or a comprehensive developer guide? Recommend: quick-start only for Phase 13; expand in a future phase.

---

## Summary

| Goal | Pre-run Status | Blocker |
|------|---------------|---------|
| G1 — Stage 10 live run | BLOCKED | Release build omits dev endpoint; GATEWAY_URL port mismatch |
| G2 — Commit `.wasm-size-baseline` | NOT STARTED | Stage 6 in DinD + wasm-pack install |
| G3 — Fix Stage 10 failures | PARTIALLY IDENTIFIED | 3 known pre-run bugs; unknown runtime failures pending live run |
| G4 — `make baseline-wasm` + `docs/DEVELOPMENT.md` | NOT STARTED | New files, no blockers |
| G5 — ESLint env-var coercion rule | NOT STARTED | ESLint absent; add from scratch |

**Recommended planning order:**

1. Fix the 3 known pre-run blockers (dev endpoint feature flag, GATEWAY_URL port,
   iggy healthcheck) — enables a clean Stage 10 first run.
2. Add Stage 6 `wasm-pack` installation — enables G2.
3. Create `Makefile` + `docs/DEVELOPMENT.md` — G4.
4. Add ESLint + custom rule — G5.
5. Run Stage 10, triage any remaining failures — G1/G3.
