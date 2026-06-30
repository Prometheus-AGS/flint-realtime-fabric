# Goals — Phase 9: Policy Wiring, SDK Hardening, and CI Integration

> Seeded from Phase 8 reflection · 2026-06-21

## Context

Phase 8 delivered the Cedar `ActionPolicyProvider` adapter, OTEL OTLP exporter, LogBroker spine
wiring, `TenantActorRegistry` config, Criterion benchmarks, Docker Compose stack, and
`signal_mux` domain tests. The structural pieces are in place but several wiring gaps remain
before the system is production-deployable.

Phase 9 closes those gaps: wire Cedar into the publish route, commit Criterion baselines,
enable Layer 3 E2E in CI, advance UniFFI Swift/Kotlin SDK generation, and run a full
workspace Clippy clean pass.

## Goals

1. **Wire Cedar into the publish route** — Add `POLICY_ENGINE` env var to `GatewayConfig`
   (`cedar` | `none`, default `none`). When `POLICY_ENGINE=cedar`, call
   `action_policy.is_permitted(tenant, "Publish", channel)` inside
   `crates/frf-gateway/src/routes/publish.rs` before broker publish. Add `frf-policy-cedar`
   to `frf-gateway/Cargo.toml` deps. Wire the concrete `CedarPolicyEngine` into `AppState`
   construction in `main.rs`.

2. **Criterion baseline commit** — Run `cargo bench -p frf-crdt -- --save-baseline main`;
   commit the `.criterion/` baseline JSON. Add a CI step in `dagger/codegen.ts` that runs
   `cargo bench -p frf-crdt -- --load-baseline main` and fails on >20% regression.

3. **Layer 3 E2E in CI** — Enable `ENABLE_INTEGRATION_STAGE=true` in the Dagger pipeline
   for the integration stage. Validate the full WASM → gateway → LogBroker → fan-out path
   against the Compose stack. Wire `WASM_AVAILABLE=1` and `GATEWAY_URL=http://localhost:8080`
   in the Playwright E2E run.

4. **UniFFI Swift/Kotlin SDK generation** — `frf-ffi` scaffolding exists from Phase 3.
   Generate the Swift and Kotlin bindings via `uniffi-bindgen generate`. Verify the Swift
   binding compiles against a minimal iOS/macOS target. Add Dagger stage for FFI codegen.

5. **Workspace Clippy clean pass** — Run `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
   and fix all remaining pedantic warnings. Establish as a hard CI gate so warnings
   cannot accumulate across future phases.

6. **Admin UI E2E Layer 2** — Wire `pnpm exec playwright test` against a live Vite dev
   server (`admin-ui/`) in the Dagger UI stage (no Docker required). Ensure `p7-smoke.spec.ts`
   and any new Layer 2 specs pass cleanly.

7. **`ReqwestMatrixClient` federation stub assessment** — Determine if Tuwunel SDK is
   available. If yes: implement the `FederationBridge` port methods. If no: document the
   explicit external dependency and mark as blocked with an issue reference.

## Carry-Forward Debt Entering Phase 9

| Debt Item | Source Phase | Work in Phase 9 |
|---|---|---|
| Cedar not wired into publish handler | Phase 8 | Goal 1 |
| `frf-policy-cedar` not a `frf-gateway` dep | Phase 8 | Goal 1 |
| `.criterion/` baseline not committed | Phase 8 | Goal 2 |
| Layer 3 E2E gated on `ENABLE_INTEGRATION_STAGE=true` | Phase 8 | Goal 3 |
| UniFFI Swift/Kotlin codegen not run | Phase 3 | Goal 4 |
| `clippy::pedantic` warnings accumulating | Phases 1–8 | Goal 5 |
| `ReqwestMatrixClient` federation impl blocked on Tuwunel | Phase 6 | Goal 7 |
