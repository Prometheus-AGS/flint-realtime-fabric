# Plan — Phase 9: Policy Wiring, SDK Hardening, and CI Integration

> Generated: 2026-06-21 · Tool: kbd-plan

---

## Open Decision Resolutions

| Decision | Resolution |
|---|---|
| `AppState` generic arity for `ActionPolicyProvider` | Add `P` as 6th generic parameter; `AppState<L,A,I,M,B,P>`. Update all 2 instantiation sites in `main.rs` and all route handler bounds. Add `NoOpPolicyProvider` to `frf-ports` as the default no-op implementation. |
| `uniffi-bindgen` binary source | Use `cargo install uniffi-bindgen@0.31.2` (matches `uniffi = "0.31.2"` workspace dep). No workspace `[[bin]]` needed; Dagger stage calls `cargo run --bin uniffi-bindgen` which requires the binary to be on PATH in the container. Fix: use `cargo install` in Dagger or wire a workspace binary crate. |
| Cedar `None` policy | `NoOpPolicyProvider` in `frf-ports/src/policy.rs` — always returns `Ok(true)`. Zero overhead at runtime; eliminates the need for an `Option<Arc<dyn ActionPolicyProvider>>` in `AppState`. |
| Matrix federation | Option A: mark `ReqwestMatrixClient` as `BLOCKED_ON_TUWUNEL` with doc comment. No new code; one-line change. |
| Criterion baseline storage | Run `cargo bench -p frf-crdt -- --save-baseline main` locally; `.criterion/` JSON committed. CI stage runs with `--load-baseline main` only when `ENABLE_BENCH_STAGE=true`. |
| Layer 3 E2E WASM availability | Dagger Stage 9 builds the WASM bundle (re-using Stage 6 output) before starting the Compose stack. Gateway container picks up `sdks/ts/frf-wasm/` via the host-mounted volume in `compose.override.yml`. |

---

## OpenSpec Backend Detected

`openspec/` directory present and `project.json` specifies `"change_backend": "openspec"`.
All changes emitted as OpenSpec proposals.

---

## Ordered Change List

### p9-c001 — `NoOpPolicyProvider` + `AppState<P>` Generic Slot

- **Dir**: `openspec/changes/p9-c001-appstate-policy-slot/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p9-c003, p9-c004, p9-c005)
- **Description**:
  1. Add `NoOpPolicyProvider` to `crates/frf-ports/src/policy.rs`:
     ```rust
     pub struct NoOpPolicyProvider;
     #[async_trait]
     impl ActionPolicyProvider for NoOpPolicyProvider {
         async fn is_permitted(&self, _: &TenantId, _: &str, _: &str) -> Result<bool, PolicyError> {
             Ok(true)
         }
     }
     ```
  2. Add `pub use policy::NoOpPolicyProvider;` to `frf-ports/src/lib.rs`.
  3. In `crates/frf-gateway/src/lib.rs`, add `P: ActionPolicyProvider` as the 6th generic:
     - `AppState<L, A, I, M, B, P>` with `pub action_policy: Arc<P>` field.
     - Update `AppStateArc<L,A,I,M,B,P>` type alias.
     - Update `build_router<L,A,I,M,B,P>` signature and all route handler bounds.
  4. Update `crates/frf-gateway/src/main.rs` — both `AppState` construction sites —
     to pass `action_policy: Arc::new(NoOpPolicyProvider)` (Cedar wiring happens in p9-c002).
  5. Update all route handlers that reference `AppStateArc<L,A,I,M,B>` to add the `P` parameter.
  6. Update `crates/frf-gateway/tests/signal_mux.rs` — `AppState` smoke test — to add the `P` slot
     (use `NoOpPolicyProvider` as the concrete type).
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-gateway -- signal_mux` passes.

---

### p9-c002 — Wire Cedar into Publish Route

- **Dir**: `openspec/changes/p9-c002-cedar-publish-wiring/`
- **Agent**: rust-reviewer
- **Deps**: p9-c001 (`AppState<P>` slot must exist)
- **Description**:
  1. Add `frf-policy-cedar = { path = "../frf-policy-cedar" }` to `crates/frf-gateway/Cargo.toml`.
  2. Add `PolicyEngineMode` enum to `crates/frf-gateway/src/config.rs`:
     ```rust
     #[non_exhaustive]
     pub enum PolicyEngineMode { None, Cedar }
     ```
     Parse from `POLICY_ENGINE` env var (`"cedar"` → `Cedar`; default `None`).
     Add `policy_engine: PolicyEngineMode` field to `GatewayConfig`.
     Add `policy_engine: PolicyEngineMode::None` to `GatewayConfig::test_default()`.
  3. In `crates/frf-gateway/src/main.rs`, after reading config:
     - If `config.policy_engine == PolicyEngineMode::Cedar`: construct `Arc::new(CedarPolicyEngine::new()?)` and use as `action_policy`.
     - Otherwise: use `Arc::new(NoOpPolicyProvider)`.
     - The gateway binary becomes `AppState<IggyBroker, ..., CedarPolicyEngine>` or `AppState<..., NoOpPolicyProvider>` — this requires either a trait object or separate code paths. Use `Box<dyn ActionPolicyProvider>` via a wrapper or use `Arc<dyn ActionPolicyProvider + Send + Sync>` type erasure for the `action_policy` field when the enum approach is chosen. **Preferred**: add a type-erased `DynPolicyProvider = Arc<dyn ActionPolicyProvider>` newtype in `frf-ports`, similar to `DynMediaSignaler`.
  4. In `crates/frf-gateway/src/routes/publish.rs`, after JWT verification and before broker publish:
     ```rust
     if !state.action_policy.is_permitted(&tenant_id, "Publish", &envelope.channel_id.to_string()).await
         .map_err(|_| AppError::Internal)? {
         return axum::http::StatusCode::FORBIDDEN.into_response();
     }
     ```
     Add `P: ActionPolicyProvider` bound to `publish_event<L,A,I,M,B,P>`.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-gateway` passes; `cargo test -p frf-policy-cedar` passes.

---

### p9-c003 — Criterion Baseline Commit + CI Bench Stage

- **Dir**: `openspec/changes/p9-c003-criterion-baseline/`
- **Agent**: devops-engineer
- **Deps**: none
- **Parallel candidate**: yes (with p9-c001, p9-c004, p9-c005)
- **Description**:
  1. Run `cargo bench -p frf-crdt -- --save-baseline main` (produces `target/criterion/crdt_merge/`).
  2. Copy baseline JSON to `.criterion/crdt_merge/` at repo root; add to `.gitignore` exclusion list (allow `.criterion/` JSON, ignore `target/criterion/`).
  3. Update `.gitignore` — add `target/criterion/` to ignored paths; ensure `.criterion/` is NOT ignored.
  4. In `dagger/codegen.ts`, add Stage 9 (`bench`) after Stage 8:
     ```typescript
     // Stage 9: Criterion bench (optional — gated on ENABLE_BENCH_STAGE=true)
     if (process.env.ENABLE_BENCH_STAGE === "true") {
         const benchStage = rustBase
             .withExec(["cargo", "bench", "-p", "frf-crdt", "--", "--load-baseline", "main"]);
         await benchStage.sync();
     }
     ```
  5. Commit `.criterion/crdt_merge/baseline.json` (the raw Criterion baseline).
- **Exit**: `.criterion/crdt_merge/baseline.json` exists and is committed; Dagger `codegen.ts` typechecks.

---

### p9-c004 — Layer 3 E2E Integration Stage in Dagger

- **Dir**: `openspec/changes/p9-c004-dagger-integration-stage/`
- **Agent**: devops-engineer
- **Deps**: none (compose.yml already exists)
- **Parallel candidate**: yes (with p9-c001, p9-c003, p9-c005)
- **Description**:
  In `dagger/codegen.ts`, add Stage 10 (`integration`) after Stage 9:
  ```typescript
  // Stage 10: Layer 3 E2E (Docker Compose + WASM) — gated on ENABLE_INTEGRATION_STAGE=true
  if (process.env.ENABLE_INTEGRATION_STAGE === "true") {
      // Start Compose stack, wait for gateway healthz, run playwright, tear down
      const integrationStage = client
          .host()
          .directory(".", { exclude: ["target/", "node_modules/"] });
      // Uses docker compose via the host socket — mount /var/run/docker.sock
      const integrationContainer = client
          .container()
          .from("mcr.microsoft.com/playwright:v1.44.0-jammy")
          .withMountedDirectory("/app", integrationStage)
          .withWorkdir("/app/admin-ui")
          .withEnvVariable("GATEWAY_URL", "http://localhost:8080")
          .withEnvVariable("WASM_AVAILABLE", "1")
          .withEnvVariable("SKIP_INTEGRATION", "false")
          .withExec(["sh", "-c",
              "docker compose -f ../compose.yml up -d && " +
              "until curl -sf http://localhost:8080/healthz; do sleep 1; done && " +
              "npx playwright test e2e/p7-smoke.spec.ts && " +
              "docker compose -f ../compose.yml down"]);
      await integrationContainer.sync();
  }
  ```
  **Note**: Full Docker-in-Docker is complex; the simpler approach is to document that this stage
  runs on a CI host with Docker available and the Compose stack is started via `--network=host`.
  The Dagger stage should be documented with a comment explaining the requirement.
- **Exit**: `dagger/codegen.ts` typechecks; `docker compose config` exits 0; stage is guarded on env var.

---

### p9-c005 — UniFFI Binding Generation + `sdks/` Scaffold

- **Dir**: `openspec/changes/p9-c005-uniffi-bindings/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p9-c001, p9-c003, p9-c004)
- **Description**:
  1. Add `uniffi-bindgen` binary to the workspace. Create `crates/uniffi-bindgen/`:
     - `Cargo.toml`: `[[bin]] name = "uniffi-bindgen"` with `uniffi = { workspace = true, features = ["cli"] }`
     - `src/main.rs`: `fn main() { uniffi::uniffi_bindgen_main() }`
  2. Add `"crates/uniffi-bindgen"` to workspace `members` in root `Cargo.toml`.
  3. Run `cargo run --bin uniffi-bindgen generate --language swift --library target/release/libfrf_ffi.dylib --out-dir sdks/swift/Sources/FrfFfi/` (macOS) or `.so` (Linux).
  4. Run `cargo run --bin uniffi-bindgen generate --language kotlin --library target/release/libfrf_ffi.so --out-dir sdks/kotlin/lib/src/main/kotlin/uniffi/frf/`.
  5. Commit the generated `frf.swift` and `frf.kt` files.
  6. Add minimal `sdks/swift/Package.swift` scaffold pointing to the generated Swift source.
  7. Add minimal `sdks/kotlin/build.gradle.kts` scaffold.
  8. Update Dagger Stages 2–3 in `codegen.ts` to use `cargo run --bin uniffi-bindgen` instead of `cargo run --bin uniffi-bindgen --` (ensure the binary crate is on PATH via the container's `cargo build --bin uniffi-bindgen`).
- **Exit**: `cargo build --bin uniffi-bindgen` succeeds; `sdks/swift/Sources/FrfFfi/frf.swift` exists; `cargo check --workspace` passes.

---

### p9-c006 — Clippy CI Gate + Matrix Blocker Doc

- **Dir**: `openspec/changes/p9-c006-clippy-gate-matrix-blocker/`
- **Agent**: rust-reviewer
- **Deps**: none — can be last, or parallel
- **Parallel candidate**: yes
- **Description**:
  1. **Clippy CI gate**: In `dagger/codegen.ts` Stage 1 (rust-build), add `RUSTFLAGS="-D warnings -W clippy::pedantic"` env var to the `cargo clippy` invocation (currently only `-D warnings`). Confirm no pedantic warnings remain (already verified as 0).
  2. **Matrix blocker**: In `crates/frf-bridge-matrix/src/client.rs`, add a `// BLOCKED_ON_TUWUNEL` doc comment to `ReqwestMatrixClient`:
     ```rust
     /// REST-based Matrix client using the Client-Server API.
     ///
     /// **BLOCKED**: This stub polls `/sync` for room events. Production implementation
     /// requires the Tuwunel Matrix homeserver SDK (https://github.com/girlbossceo/conduwuit),
     /// which is not yet available as a stable Rust crate. This stub is intentionally
     /// incomplete and will be replaced when Tuwunel publishes a client SDK.
     ///
     /// Tracking: see project backlog "Phase 9 Carry-Forward — Matrix federation".
     ```
  3. Run `cargo check --workspace` to confirm no regressions from the doc-comment addition.
- **Exit**: `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0 (already true); doc comment committed; `codegen.ts` typechecks.

---

## Dependency Graph

```
p9-c001 (AppState<P> slot)
    │
    ▼
p9-c002 (Cedar publish wiring)   ← depends on p9-c001

p9-c003 (Criterion baseline)      ─── parallel, no consumers
p9-c004 (Dagger integration stage) ── parallel, no consumers
p9-c005 (UniFFI bindings)         ─── parallel, no consumers
p9-c006 (Clippy gate + Matrix doc) ── parallel, can run last
```

p9-c001 and p9-c003/c004/c005/c006 are fully parallel.
p9-c002 depends on p9-c001.

---

## Quality Gate Protocol

After each Rust change:
1. `cargo check --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
3. `cargo test -p <affected-crate>`

After p9-c002:
- Verify `POLICY_ENGINE=cedar cargo run -p frf-gateway` starts without panic (env-gated Cedar construction)
- Verify `POLICY_ENGINE=none cargo run -p frf-gateway` also starts cleanly

After p9-c005:
- `cargo build --bin uniffi-bindgen --release` succeeds
- `sdks/swift/Sources/FrfFfi/frf.swift` present

---

## Phase Exit Criterion

**SATISFIED when**:
- `cargo check --workspace` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
- `cargo test -p frf-gateway -- signal_mux` passes
- `cargo test -p frf-policy-cedar` passes (3/3)
- `POLICY_ENGINE=cedar` env var triggers Cedar construction in gateway binary
- `sdks/swift/Sources/FrfFfi/frf.swift` committed
- `.criterion/crdt_merge/baseline.json` committed
- `docker compose config` exits 0
- `dagger/codegen.ts` typechecks (`pnpm -C dagger exec tsc --noEmit`)
