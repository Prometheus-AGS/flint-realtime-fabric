# Assessment — Phase 8: Observability, Integration Tests, and Production Hardening

> Generated: 2026-06-21 · Tool: kbd-assess

---

## Codebase Scan Summary

Phase 7 delivered a complete architectural stack — sovereign SFU, WASM browser SDK, bidi
`RunAgent` gRPC, admin-UI transport toggle, and Dagger CI with Node 24. Phase 8 focuses on
making that stack production-worthy: systematic observability, spine wiring completeness,
full-stack E2E tests, performance baselines, and Cedar policy scaffolding.

---

## Gap Analysis by Goal

### Goal 1: Full-Stack E2E Integration Tests (Layer 3 + Docker Compose)

**Status: LAYER 1 COMPLETE — LAYER 3 CONDITIONAL ON WASM BUILD; NO DOCKER COMPOSE**

`admin-ui/e2e/p7-smoke.spec.ts` was written as Phase 7's exit gate:
- Layer 1 (UI shape, no gateway) — 8 tests, all static, run in CI.
- Layer 2 (WS signaling, gated by `SKIP_INTEGRATION`) — 4 tests, require live gateway.
- Layer 3 (gRPC/WASM, gated by `WASM_AVAILABLE=1`) — 1 test, requires full stack + WASM build.

There is **no `docker-compose.yml`** at the repo root or anywhere in the workspace. The
integration test skeleton at `crates/frf-gateway/tests/subscribe_mux.rs` references a
`docker-compose.yml` in a comment but no such file exists.

The Dagger CI pipeline (Stage 8) runs `p7-smoke.spec.ts` with no `GATEWAY_URL` set, so
Layers 2 and 3 are always skipped in CI. Layer 3 has never been exercised.

**Work needed:**
1. Add `compose.yml` at project root with services: gateway, Apache Iggy, Ory Keto, Ory
   Oathkeeper, Ory Kratos, SurrealDB, PostgreSQL.
2. Add a Dagger integration stage that starts the compose stack and runs `p7-smoke.spec.ts`
   with `GATEWAY_URL=http://localhost:8080 WASM_AVAILABLE=1`.
3. Wire the Dagger WASM build output into the Layer 3 browser test so the `import()` resolves.

---

### Goal 2: Observability — `#[instrument]` on Port Adapters + OpenTelemetry

**Status: PARTIAL — Core Use-Cases Instrumented; Adapters Inconsistent; No OTEL Exporter**

Confirmed instrumented:
- `frf-app/src/publish.rs` — `#[instrument(name = "app::publish", ...)]` ✓
- `frf-app/src/subscribe.rs` — `#[instrument(name = "app::subscribe", ...)]` ✓
- `frf-gateway/src/routes/publish.rs` — `#[instrument(name = "http::publish", ...)]` ✓
- `frf-gateway/src/routes/subscribe.rs` — `#[instrument(name = "ws::subscribe", ...)]` ✓

Missing instrumentation (confirmed by scan):
- `crates/frf-authz-keto/src/provider.rs` — `use tracing::instrument;` imported but
  individual methods not confirmed to have `#[instrument]` decorators.
- `crates/frf-identity-ory/src/verifier.rs` — same.
- `crates/frf-crdt/src/store.rs` — no `#[instrument]` confirmed.
- `crates/frf-store-redb/src/lib.rs` — no `#[instrument]` confirmed.
- `crates/frf-store-surreal/src/lib.rs` — no `#[instrument]` confirmed.
- `crates/frf-broker-iggy/src/lib.rs` — no `#[instrument]` confirmed.

`frf-ports/src/authz.rs` and `frf-ports/src/identity.rs` explicitly state:
> "Adapter crates MUST instrument methods with `#[tracing::instrument]`."

This requirement is not yet universally enforced.

No `opentelemetry*` crates in workspace Cargo.toml. Only `tracing` and `tracing-subscriber`
are present. Span export to a collector is not wired.

**Work needed:**
1. Audit each adapter crate (`frf-authz-keto`, `frf-identity-ory`, `frf-crdt`, `frf-store-*`,
   `frf-broker-iggy`) — add `#[instrument]` to every public port-boundary method.
2. Add `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry` to workspace deps.
3. In `frf-gateway/src/main.rs`, init the OTEL tracer provider (OTLP gRPC exporter, env-driven
   `OTEL_EXPORTER_OTLP_ENDPOINT`) and register `tracing_opentelemetry::layer()`.

---

### Goal 3: LogBroker Spine Wiring (`/dev/inject-federation-event`)

**Status: STUB — 202 Only, No Publish**

Confirmed state of `crates/frf-gateway/src/routes/dev.rs` (handler `inject_federation_event`):
```rust
pub async fn inject_federation_event(
    Json(body): Json<InjectFederationEventRequest>,
) -> impl IntoResponse {
    tracing::debug!(...);
    StatusCode::ACCEPTED
}
```

No `State(state)` parameter. No `log_broker` call. The handler is a pure acknowledgement stub.
`AppState` has a `log_broker: Arc<L>` field (confirmed by gateway route patterns), but the
inject handler was never wired to it.

This blocks Layer 3 smoke tests: the test injects a signal and expects it to propagate through
the spine to the subscriber, but nothing is published.

**Work needed:**
1. Add `State(state): State<AppStateArc<L, A, I, M, B>>` parameter to the handler (matching
   generic signature of other debug routes like `inject_signal`).
2. Parse `body` into a domain `EventEnvelope` (federation `channel_id` derived from protocol +
   source, `tenant_id` from request body UUID, payload from `body.payload`).
3. Call `state.log_broker.publish(envelope).await`.
4. Return `202` on success, `503` on broker error.

---

### Goal 4: `SpineSignalService` Live Tonic Integration Test

**Status: SKELETON — Body Not Implemented; No Docker Compose**

`crates/frf-gateway/tests/subscribe_mux.rs` has an `#[ignore]` test with only comments:
```rust
#[tokio::test]
#[ignore = "requires live infrastructure (Iggy, Keto, Oathkeeper)"]
async fn subscriber_receives_published_event() {
    // Integration smoke test outline (comments only)
    println!("integration smoke test: infrastructure not available in CI — skipped");
}
```

The test is syntactically valid and passes trivially (it just prints a message). Nothing is
actually wired.

For `SpineSignalService` specifically, the gap is:
- The service type exists at `crates/frf-gateway/src/signal_service.rs`.
- No test spins up a `tonic::transport::Server` in-process and exercises a signal exchange.

A minimal integration test does not require Iggy/Keto — it can use in-memory mock adapters
(which `frf-app` tests already demonstrate).

**Work needed:**
1. Add a `#[cfg(test)]` module to `crates/frf-gateway/src/signal_grpc_service.rs` (or a new
   `crates/frf-gateway/tests/signal_mux.rs`).
2. Spin up a `tonic::transport::Server` bound to a random localhost port using in-memory
   mock adapters (no real Iggy required).
3. Call `SignalService::send_signal` via the client channel, assert the response arrives.
4. Guard behind `#[cfg(feature = "integration-tests")]` or `#[ignore]` with a clear env flag
   (`ENABLE_INTEGRATION_TESTS=1`).

---

### Goal 5: Performance Benchmarks (Criterion)

**Status: NOT STARTED**

Confirmed absence:
- Zero `[[bench]]` sections in any crate Cargo.toml.
- No `benches/` directories anywhere in the workspace.
- `criterion` not in workspace `[dependencies]`.

Priority targets for benchmarks:
- `frf-crdt` — `crdt_apply_delta` latency (Loro merge throughput).
- `frf-broker-iggy` — publish throughput (events/sec).
- `frf-authz-keto` — Keto check round-trip latency (with a mock HTTP server).

**Work needed:**
1. Add `criterion = { workspace = true }` to workspace deps (version `0.5`).
2. Add `crates/frf-crdt/benches/crdt_merge.rs` — benchmark `apply_delta` for 100-op and
   1000-op doc histories.
3. Add `[[bench]]` entries to `crates/frf-crdt/Cargo.toml`.
4. Optionally: `crates/frf-broker-iggy/benches/publish_throughput.rs` using a mock broker.

---

### Goal 6: Cedar Policy Engine (`frf-policy-cedar`)

**Status: NOT STARTED — Crate Missing; Port Trait Awaits Implementation**

Confirmed:
- `crates/frf-policy-cedar/` does not exist.
- `frf-authz-keto` implements `AuthzProvider` for visibility (Zanzibar ReBAC).
- Cedar's role is action-level mutation policy — distinct from Keto. CLAUDE.md is explicit:
  > "Cedar governs action policy (mutating ops), not visibility. Do not conflate with Keto."

The `AuthzProvider` port already defines `check_permission`, `write_relation`,
`delete_relation` — Cedar would implement a subset (action checks only, not relation writes).
A new port trait (`ActionPolicyProvider`) may be more appropriate to avoid conflation.

**Work needed:**
1. Define `ActionPolicyProvider` trait in `frf-ports/src/policy.rs` (separate from `AuthzProvider`).
2. Create `crates/frf-policy-cedar/Cargo.toml` and `src/lib.rs`.
3. Add `cedar-policy = "4"` to workspace deps (verify current version).
4. Implement `ActionPolicyProvider` using Cedar's in-memory policy store.
5. Define sample policy set: `permit(principal, action == Action::"Publish", resource) when { ... }`.
6. Wire into `frf-gateway` behind `POLICY_ENGINE=cedar` env var.

---

### Goal 7: `TenantActorRegistry` Sweep Interval Configurable

**Status: IDLE TIMEOUT CONFIGURABLE; SWEEP INTERVAL HARDCODED AT 60s**

`crates/frf-librefang/src/registry.rs` line ~120:
```rust
let mut interval = tokio::time::interval(Duration::from_secs(60));
```

`GatewayConfig` (`crates/frf-gateway/src/config.rs`) does not have a
`registry_sweep_interval_secs` field.

**Work needed:**
1. Add `registry_sweep_interval_secs: u64` to `GatewayConfig` (env: `REGISTRY_SWEEP_INTERVAL_SECS`, default 60).
2. Add `registry_idle_secs: u64` to `GatewayConfig` (env: `REGISTRY_IDLE_SECS`, default 300) — the idle timeout is currently passed as a magic number in `main.rs`.
3. Thread both values from `GatewayConfig` into the registry spawn call.
4. Update `spawn_eviction_task(idle_secs: u64)` signature to also accept `sweep_interval_secs: u64`.

---

## Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| Docker Compose service count (7+ services) makes local dev heavy | MEDIUM | Add `compose.minimal.yml` with just gateway + in-memory mock broker |
| OpenTelemetry SDK adds compile-time and binary size | LOW | Gate behind `otel` Cargo feature; disable in dev by default |
| Cedar 4.x API churn | MEDIUM | Pin exact version; thin adapter layer; add breaking-change CI check |
| Criterion benchmarks require stable hardware baselines | LOW | Store results as committed `.criterion/` snapshots; alert on >20% regression |
| LogBroker spine wiring may expose Iggy connection errors in dev | LOW | Return 503 with `{ "error": "broker_unavailable" }` body |

---

## Open Decisions (Resolve Before Plan)

| Decision | Impact | Recommendation |
|---|---|---|
| Docker Compose approach: monolithic vs minimal | Blocks Goal 1 | Start with `compose.yml` (full stack) + `compose.override.yml` for minimal dev |
| OTEL exporter: OTLP vs stdout | Blocks Goal 2 | OTLP gRPC default; stdout as fallback when `OTEL_EXPORTER_OTLP_ENDPOINT` unset |
| Cedar port design: reuse `AuthzProvider` vs new `ActionPolicyProvider` | Blocks Goal 6 | New `ActionPolicyProvider` in `frf-ports` — keeps Cedar's mutation semantics separate from Keto's visibility semantics |
| Criterion baseline storage: committed snapshots vs CI artifact | Blocks Goal 5 | Commit `.criterion/` baselines; fail CI on >20% regression |

---

## Dependency Order

```
Goal 7 (registry config)          Goal 3 (spine wiring)
    │                                   │
    │                    ┌──────────────┘
    ▼                    ▼
Goal 2 (observability) ─────────────────────────────────────── parallel
    │
    ├─ Goal 5 (benchmarks) ──── parallel to all
    │
    └─ Goal 4 (tonic test) ─── no Docker required; can run early
                │
                ▼
Goal 1 (full-stack E2E) ← depends on Docker Compose + LogBroker wiring
    │
    └─ Goal 6 (Cedar) ── last; no other goal depends on it
```

Goals 2, 4, 5, 7 are parallelizable. Goal 1 is the sync point requiring Goals 3 and the Docker
Compose. Goal 6 (Cedar) is independent and lowest priority.

---

## Planned Change Count: 8 changes

- p8-c001: LogBroker spine wiring for `/dev/inject-federation-event`
- p8-c002: `#[instrument]` audit across all adapter crates
- p8-c003: OpenTelemetry exporter wiring in `frf-gateway`
- p8-c004: `SpineSignalService` in-process tonic integration test
- p8-c005: `TenantActorRegistry` sweep + idle interval configurability
- p8-c006: Criterion benchmarks for `frf-crdt` and `frf-broker-iggy`
- p8-c007: Docker Compose stack + Dagger integration stage
- p8-c008: `frf-policy-cedar` — `ActionPolicyProvider` port + Cedar adapter crate

---

## Assessment Verdict

Phase 8 is **buildable** with no blocking external unknowns. All dependencies (Cedar 4.x,
`opentelemetry-otlp`, `criterion 0.5`) are stable published crates. The main effort is
systematic: wiring what was already designed (observability, spine paths) and adding
infrastructure that was deferred (Docker Compose, benchmarks, Cedar). No architectural
reversals are required.
