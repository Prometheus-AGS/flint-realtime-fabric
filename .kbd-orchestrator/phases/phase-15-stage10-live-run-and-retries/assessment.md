# Assessment — phase-15-stage10-live-run-and-retries

> Authored: 2026-06-30
> Assessor: kbd-assess

---

## Summary

Phase 14 delivered 5 of 6 automatable changes. The one remaining manual
operation (G2 — `.wasm-size-baseline`) and the deferred decision (G4 —
`--retries=2`) depend on a live Stage 10 run. Assessment reveals one
**definite pre-run bug** that will prevent Stage 10 from succeeding without
a fix, and four **risk areas** to monitor during the live run.

---

## Codebase State Inspection

### 1. compose.yml

**Status: CORRECT**

- `gateway` service builds with `CARGO_FEATURES: dev-endpoints`
- `DEV_NO_AUTH: "true"` present in gateway env
- `GATEWAY_JWKS_URL: "http://flint-gate:4457/signing-keys"` — correct endpoint
- `flint-gate` service builds from `/Users/gqadonis/Projects/prometheus/flint-gate`
- flint-gate healthcheck uses TCP probe (no curl dependency in minimal image)
- Gateway depends on iggy + keto + postgres with health conditions

### 2. dagger/codegen.ts — Stage 10 block

**Status: ONE CRITICAL BUG**

The Stage 10 Dagger container polls:
```
for i in $(seq 1 30); do curl -sf http://localhost:8080/healthz && break || sleep 2; done
```

But `compose.yml` maps the gateway port as `28080:8080` (host:container). In
the DinD environment the Dagger runner sees the Docker host network; the
gateway is reachable at `localhost:28080`, not `localhost:8080`.

**This will cause Stage 10 to time out and fail immediately after `docker compose up -d`.**

Additionally: `GATEWAY_URL=http://localhost:8080` is passed to Playwright.
The Phase 4/5 Layer 3 E2E tests POST to `${GATEWAY_URL}/v1/publish` — these
will also fail to reach the gateway for the same reason.

**Fix required: Change the healthz poll and GATEWAY_URL to use port 28080.**

### 3. flint-gate Dockerfile

**Status: POTENTIALLY RISKY — needs first-build verification**

The Dockerfile:
- Multi-stage: `rust:1.82-bookworm` builder → `debian:bookworm-slim` runtime
- Dependency-caching layer: creates stub `main.rs`/`lib.rs`, then runs `cargo
  build --release 2>/dev/null || true`
- Source build: `COPY crates ./crates` → `touch` → `cargo build --release`

**Concern**: The Dockerfile assumes `Cargo.lock` exists in the flint-gate repo
root. If `Cargo.lock` is absent or gitignored, the COPY step will fail with a
build error before the dependency cache even starts. The flint-gate
`crates/` directory exists (`flint-gate`, `flint-gate-core`,
`flint-gate-client`) confirming the workspace structure matches the Dockerfile.

**Concern**: The `--release` binary name is `flint-gate`. If the binary name
in the workspace differs from the crate name, the `COPY --from=builder` line
will fail. This needs to be verified by the live build.

**Risk level: MEDIUM** — cannot verify without actually running the build. If
the Dockerfile fails, it blocks all of Stage 10. The fix, if needed, lives in
the flint-gate repo (not FRF).

### 4. subscribe.rs — DEV_NO_AUTH bypass

**Status: CORRECT**

Both `#[cfg(feature = "dev-endpoints")]` and `#[cfg(not(feature =
"dev-endpoints"))]` paths are implemented correctly. With `DEV_NO_AUTH=true`:
- Subscribe upgrade proceeds with `bearer_token = String::new()`
- The identity verifier receives an empty token — behavior depends on
  `frf-identity-ory`'s handling of empty tokens when `DEV_NO_AUTH=true`

**Risk**: The identity verifier (`frf-identity-ory`) may still attempt to
validate an empty JWT against the JWKS URL and return an error, even though
`DEV_NO_AUTH` bypasses the token extraction. The bypass short-circuits before
`SubscribeRequest` uses the token for authz, but if the use-case layer still
calls the identity verifier with the empty string, a 401 will surface.

This is a **medium risk** that only a live run can confirm.

### 5. phase6-smoke.spec.ts — c003/c004/c005

**Status: CORRECT**

- `ENABLE_FEDERATION` gate present (L143–L145)
- `DEV_ENDPOINTS_ENABLED` skip guard present (L104–L107)
- `tenant_id` present in both Layer 3 inject POST bodies (L159, L179)

### 6. Phase 4/5 E2E specs

**Status: AUTH ISSUE WILL SURFACE IF PORT BUG IS FIXED**

- `phase4-smoke.spec.ts` Layer 3 posts to `/v1/publish` with no Authorization
  header — this is correct (DEV_NO_AUTH bypasses the requirement)
- `layer2-subscribe.spec.ts` Layer 2 opens WebSocket with no Authorization
  header — this is correct per subscribe.rs DEV_NO_AUTH path

No auth-specific bugs in Phase 4/5 specs. The port bug (finding #2) is
the blocker — fix it first.

### 7. config.rs test_default

**Status: MINOR NOTE**

The `test_default()` still has the old JWKS URL pattern
(`http://localhost:4456/.well-known/jwks.json`). This is used only in
unit/integration tests (not the live compose stack), so it does not affect
Stage 10. Low priority.

---

## Gap Analysis Against Goals

| Goal | Status | Gap / Blocker |
|------|--------|---------------|
| G1 — Stage 10 live run | **BLOCKED** | Port 8080 vs 28080 in dagger Stage 10 healthz poll + GATEWAY_URL |
| G2 — `.wasm-size-baseline` commit | **MANUAL — PENDING** | Depends on G1 succeeding (Stage 6 WASM build) |
| G3 — Triage new runtime failures | **NOT STARTED** | Depends on G1 |
| G4 — Playwright `--retries=2` | **NOT STARTED** | Depends on G1 flakiness data |
| G5 — flint-gate Dockerfile verify | **UNVERIFIED** | Needs live DinD compose build; Cargo.lock presence unconfirmed |

---

## Change Requirements for This Phase

### p15-c001 — Fix dagger Stage 10 port references (CRITICAL)

- Change healthz poll in `dagger/codegen.ts` from `localhost:8080` to `localhost:28080`
- Change `GATEWAY_URL` from `http://localhost:8080` to `http://localhost:28080`

**Files**: `dagger/codegen.ts`
**Risk**: NONE — pure port number change, verifiable without a live run

### p15-c002 — Verify and fix flint-gate Dockerfile if needed (conditional)

Assessment: The Dockerfile structure looks correct structurally. The live
compose build will be the real test. If it fails:
- Check if `Cargo.lock` exists in flint-gate repo
- Check if binary output name matches `flint-gate`

**Files**: `/Users/gqadonis/Projects/prometheus/flint-gate/Dockerfile` (if fix needed)
**Risk**: LOW — Dockerfile is correct structurally; fix only if build fails

### p15-c003 — Run Stage 10 live (manual trigger, DinD required)

After p15-c001: operator runs
```bash
ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```
from a DinD-capable environment. Capture output and triage failures in G3.

### p15-c004 — Post-run: wire `--retries=2` if flakiness observed (conditional)

After G1 live run output: if ≥1 test category shows intermittent failure,
add `--retries=2` to the Stage 10 Playwright invocation.

**Files**: `dagger/codegen.ts`

---

## Open Questions

1. **Is `Cargo.lock` committed in the flint-gate repo?** The Dockerfile
   references it in `COPY Cargo.toml Cargo.lock ./` — if absent the build
   fails before the first layer.

2. **Does `frf-identity-ory` handle empty bearer tokens gracefully when
   `DEV_NO_AUTH=true`?** If the use-case passes the empty string to the
   identity verifier, subscribe WS upgrades will fail with 401.

3. **Will CDC timing cause flakiness?** The replication slot must be active
   before the CDC smoke tests poll. The gateway starts after postgres becomes
   healthy, but the CDC listener startup adds latency.

---

## Recommended Plan Order

1. **p15-c001** (fix port in dagger) — blocker, do immediately
2. **p15-c003** (live Stage 10 run) — manual, requires DinD
3. **p15-c002** (flint-gate Dockerfile fix, if needed) — conditional on c003
4. **p15-c004** (retries, if flakiness observed) — conditional on c003
5. **G2** (`.wasm-size-baseline`) — manual, after Stage 6 completes in c003

---

## Verdict

One automatable fix (p15-c001) must land before Stage 10 has any chance of
succeeding. Everything else is conditional on the live run output.
