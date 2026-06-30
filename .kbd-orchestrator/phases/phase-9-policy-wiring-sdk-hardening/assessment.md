# Assessment — Phase 9: Policy Wiring, SDK Hardening, and CI Integration

> Written: 2026-06-21 · Tool: kbd-assess

---

## Codebase State at Phase Entry

| Artifact | Status |
|----------|--------|
| `cargo check --workspace` | PASS — zero errors |
| `cargo clippy --workspace -- -W clippy::pedantic` | PASS — zero pedantic warnings |
| `cargo test -p frf-policy-cedar` | PASS — 3/3 |
| `cargo test -p frf-gateway -- signal_mux` | PASS — 3/3 |
| `cargo bench -p frf-crdt` bench harness | EXISTS — `crdt_merge.rs` present; no baseline committed |
| `docker compose config` | Not yet verified (compose.yml exists at repo root) |
| `admin-ui` E2E specs | 5 spec files in `admin-ui/e2e/`; playwright.config.ts exists |

---

## Gap Analysis by Goal

### Goal 1 — Wire Cedar into the publish route

**Status: NOT STARTED**

Gaps:
- `crates/frf-gateway/Cargo.toml` — `frf-policy-cedar` dep absent
- `crates/frf-gateway/src/config.rs` — no `policy_engine` field or `POLICY_ENGINE` env var
- `crates/frf-gateway/src/routes/publish.rs` — no `action_policy.is_permitted(...)` call
- `crates/frf-gateway/src/main.rs` — `CedarPolicyEngine` not constructed or injected into `AppState`
- `crates/frf-ports/src/lib.rs` — `ActionPolicyProvider` trait exists but `AppState` generic doesn't include a policy provider slot yet (needs audit)

Work required:
1. Add `ActionPolicyProvider` generic slot `P` to `AppState<L,A,I,M,B>` in `frf-gateway/src/lib.rs`
2. Add `frf-policy-cedar` dep to `frf-gateway/Cargo.toml`
3. Add `policy_engine: PolicyEngineMode` field to `GatewayConfig` (enum: `Cedar | None`)
4. Construct `CedarPolicyEngine` in `main.rs` when `POLICY_ENGINE=cedar`
5. In `publish.rs` handler, call `state.action_policy.is_permitted(tenant, "Publish", channel).await` and return 403 on deny

---

### Goal 2 — Criterion baseline commit

**Status: NOT STARTED**

Gaps:
- `.criterion/` directory does not exist (no baseline saved yet)
- `cargo bench -p frf-crdt -- --save-baseline main` has not been run
- Dagger `codegen.ts` has no `cargo bench` step (Stage 9 slot is available — currently 8 stages)

Work required:
1. Run `cargo bench -p frf-crdt -- --save-baseline main` to produce `target/criterion/`
2. Copy/symlink baseline JSON to `.criterion/` and commit
3. Add Dagger Stage 9 (`bench`) gated on `ENABLE_BENCH_STAGE=true` that runs bench + baseline comparison

---

### Goal 3 — Layer 3 E2E in CI

**Status: SCAFFOLDED, NOT WIRED**

Present:
- `compose.yml` — full stack (gateway, iggy, keto, oathkeeper, surrealdb, postgres)
- `admin-ui/e2e/p7-smoke.spec.ts` — Layer 1 smoke exists
- Dagger Stage 8 — runs playwright `p7-smoke.spec.ts` against dev server (no Docker)

Gaps:
- Dagger has no Stage 9 (integration) that starts `docker compose up -d` and runs with `WASM_AVAILABLE=1`
- `ENABLE_INTEGRATION_STAGE=true` env var check not present in `codegen.ts`
- WASM build output needs to be available to the gateway container at runtime

Work required:
1. Add Dagger Stage 9 (`integration`) in `codegen.ts` — `docker compose up -d`, wait healthz, run playwright with `WASM_AVAILABLE=1`, `docker compose down`
2. Guard on `process.env.ENABLE_INTEGRATION_STAGE === "true"`

---

### Goal 4 — UniFFI Swift/Kotlin SDK generation

**Status: PARTIALLY DONE — Dagger stages exist; actual generated bindings not in `sdks/`**

Present:
- `crates/frf-ffi/src/lib.rs` — `uniffi::setup_scaffolding!("frf")` wired
- `crates/frf-ffi/src/crdt.rs` — 3 functions exported with `#[uniffi::export]`
- Dagger Stage 2 (uniffi-swift) and Stage 3 (uniffi-kotlin) — defined in `codegen.ts`
- Stage 2/3 read `target/release/libfrf_ffi.so` and write to `sdks/swift/` and `sdks/kotlin/`

Gaps:
- `sdks/swift/` and `sdks/kotlin/` directories contain no generated binding files yet
- `uniffi-bindgen` binary is referenced but not explicitly declared as a `[[bin]]` in workspace (needs check)
- The Dagger stages have never been run; generated bindings unverified
- No Swift Package Manager `Package.swift` or Kotlin Gradle wrapper in `sdks/swift/` / `sdks/kotlin/`

Work required:
1. Verify `uniffi-bindgen` binary exists (typically `cargo install uniffi-bindgen` or `cargo run --bin uniffi-bindgen`)
2. Run Dagger Stage 2/3 locally or add `cargo run --bin uniffi-bindgen generate` instructions
3. Commit generated `frf.swift` and `frf.kt` binding files
4. Add minimal `Package.swift` and `build.gradle.kts` scaffold for consuming the bindings

---

### Goal 5 — Workspace Clippy clean pass

**Status: ALREADY CLEAN — zero pedantic warnings**

`cargo clippy --workspace --all-targets -- -W clippy::pedantic` exits 0 with zero warnings.
This goal is satisfied at phase entry. Work for this change is to:
1. Add `RUSTFLAGS="-D warnings -W clippy::pedantic"` to `dagger/codegen.ts` `rust-check` stage (currently only `-D warnings`)
2. Confirm the gate is enforced in CI so it cannot regress

---

### Goal 6 — Admin UI E2E Layer 2

**Status: LAYER 1 EXISTS; LAYER 2 NOT DEFINED**

Present:
- `admin-ui/e2e/p7-smoke.spec.ts`, `phase4-smoke.spec.ts`, `phase5-smoke.spec.ts`, `phase6-smoke.spec.ts`, `signaling.spec.ts`
- `admin-ui/playwright.config.ts` exists
- Dagger Stage 8 runs `p7-smoke.spec.ts` against Vite dev server

Gaps:
- No "Layer 2" tests that exercise real UI interactions (form fills, WebSocket connects, route transitions)
- Stage 8 uses `SKIP_INTEGRATION=true` — the smoke test set is minimal
- `signaling.spec.ts` may require a live gateway; needs assessment

Work required:
1. Audit `signaling.spec.ts` to confirm it can run against Vite mock or requires gateway
2. Add Layer 2 tests for publish form, subscribe stream, and agent session UI flow
3. Gate Layer 2 tests on `SKIP_INTEGRATION=false` (run with Compose stack, skip by default)

---

### Goal 7 — `ReqwestMatrixClient` federation assessment

**Status: STUB EXISTS — Tuwunel SDK not available**

Present:
- `crates/frf-bridge-matrix/src/client.rs` — `ReqwestMatrixClient` polls `/sync` (REST stub)
- `MatrixClient` trait fully defined
- `MatrixBridge` `FederationBridge` adapter implemented

Assessment:
- Tuwunel (Matrix homeserver SDK) does not appear in `Cargo.toml` dependencies
- The stub uses `reqwest` polling which is functionally incomplete for production use
- No Tuwunel crate exists on crates.io as of knowledge cutoff; this remains an external dependency

Decision required before Phase 9 execution:
- **Option A**: Mark `ReqwestMatrixClient` as `BLOCKED_ON_TUWUNEL`; add doc comment; create tracking issue
- **Option B**: Implement a more complete polling `/sync` client using the Matrix Client-Server API (reqwest only, no Tuwunel)
- **Recommendation**: Option A — document the blocker clearly; do not invest in a polling stub that will be replaced

---

## Priority Order for Phase 9 Changes

| Priority | Goal | Rationale |
|----------|------|-----------|
| P1 | Goal 1 — Cedar publish wiring | Highest value; completes Phase 8 debt |
| P2 | Goal 4 — UniFFI binding generation | Phase 3 debt; unblocks mobile SDK consumers |
| P3 | Goal 3 — Layer 3 E2E in CI | Infrastructure ready; just needs Dagger wiring |
| P4 | Goal 2 — Criterion baseline | Low effort; enables perf regression guard |
| P5 | Goal 5 — Clippy CI gate | Already clean; just needs CI enforcement |
| P6 | Goal 6 — Admin UI Layer 2 | Nice-to-have; existing E2E coverage is adequate |
| P7 | Goal 7 — Matrix federation | Decision: mark blocked on Tuwunel |

---

## Open Questions for Plan

1. **`AppState` generic arity** — Does adding a `P: ActionPolicyProvider` slot break the existing `AppState<L,A,I,M,B>` consumers (tests, main.rs)? Need to audit all instantiation sites before changing the generic signature.
2. **`uniffi-bindgen` binary source** — Is it `cargo install uniffi-bindgen` or a workspace `[[bin]]`? The Dagger stage references `cargo run --bin uniffi-bindgen` which implies a workspace binary entry.
3. **Cedar `None` policy** — When `POLICY_ENGINE=none`, what concrete type satisfies `ActionPolicyProvider`? Need a `NoOpPolicyProvider` in `frf-ports` or `frf-policy-cedar`.
4. **Layer 3 WASM availability** — The Compose gateway container needs the WASM bundle available. How is it mounted? Build artifact from Dagger Stage 6 or baked into the Docker image?
