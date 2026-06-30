# Plan — Phase 8: Observability, Integration Tests, and Production Hardening

> Generated: 2026-06-21 · Tool: kbd-plan

---

## Open Decision Resolutions

| Decision | Resolution |
|---|---|
| OTEL exporter | OTLP gRPC default (`OTEL_EXPORTER_OTLP_ENDPOINT`); stdout fallback when env unset |
| Cedar port design | New `ActionPolicyProvider` trait in `frf-ports/src/policy.rs` — separate from `AuthzProvider`; Cedar governs action-level mutation checks only |
| Docker Compose approach | `compose.yml` (full stack) at repo root; `compose.override.yml` for minimal dev (gateway + mock in-memory broker) |
| Criterion baseline storage | Committed `.criterion/` baselines at `benches/baselines/`; CI fails on >20% regression |
| OTEL crate version | `opentelemetry = "0.27"`, `opentelemetry-otlp = "0.27"`, `tracing-opentelemetry = "0.28"` — current stable |

---

## Ordered Change List

### p8-c001 — LogBroker Spine Wiring (`/dev/inject-federation-event`)

- **Dir**: `openspec/changes/p8-c001-spine-wiring/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p8-c002, p8-c004, p8-c005)
- **Description**: Wire `inject_federation_event` handler in `crates/frf-gateway/src/routes/dev.rs`
  to the LogBroker. Add `State(state): State<AppStateArc<L, A, I, M, B>>` parameter (matching
  the generic signature used by `inject_signal`). Parse `InjectFederationEventRequest` body into
  a domain `EventEnvelope` — `channel_id` derived from `body.protocol + ":" + body.source`,
  `tenant_id` from `body.tenant_id.parse::<Uuid>().map(TenantId::from_uuid)`, payload from
  `body.payload`. Call `state.log_broker.publish(envelope).await`. Return 202 on success,
  400 on bad UUID parse, 503 on broker error. Update generic bounds on the route registration
  in `frf-gateway/src/lib.rs`.
- **Exit**: `cargo check --workspace` passes; `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes; `cargo test -p frf-gateway` passes.

---

### p8-c002 — `#[instrument]` Audit Across All Adapter Crates

- **Dir**: `openspec/changes/p8-c002-instrument-audit/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p8-c001, p8-c004, p8-c005)
- **Description**: Add `#[tracing::instrument(skip(self, ...))]` to every public port-boundary
  method in: `frf-authz-keto` (check, write_relation, delete_relation), `frf-identity-ory`
  (verify_jwt, verify_session), `frf-crdt` (apply_delta, merge, store_op), `frf-store-redb`
  (read, write, flush), `frf-store-surreal` (upsert, query, delete), `frf-broker-iggy`
  (publish, subscribe, ack). The `skip(self, ...)` should omit large byte payloads. Add a
  `fields(tenant_id = ...)` annotation where a `TenantId` is available. Confirm `tracing`
  import in each crate's `[dependencies]`. No behavior changes — instrumentation only.
- **Exit**: `cargo check --workspace` passes; `cargo clippy` passes; each affected crate
  compiles and tests pass.

---

### p8-c003 — OpenTelemetry Exporter Wiring

- **Dir**: `openspec/changes/p8-c003-otel-exporter/`
- **Agent**: rust-reviewer
- **Deps**: p8-c002 (instrument decorators must be in place before wiring the exporter)
- **Description**: Add `opentelemetry = "0.27"`, `opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }`, `tracing-opentelemetry = "0.28"`, `opentelemetry_sdk = "0.27"` to workspace `[dependencies]`. In `crates/frf-gateway/src/main.rs`, add an `init_telemetry()` function:
  - If `OTEL_EXPORTER_OTLP_ENDPOINT` is set, build OTLP gRPC tracer provider; register
    `tracing_opentelemetry::layer()` on the subscriber.
  - Otherwise, fall back to `tracing_subscriber::fmt` (current behavior).
  - Attach service name `frf-gateway` and version from `CARGO_PKG_VERSION`.
  - Call `opentelemetry::global::shutdown_tracer_provider()` on shutdown.
  - Add `OTEL_EXPORTER_OTLP_ENDPOINT` and `OTEL_SERVICE_NAME` to documented env vars in
    `crates/frf-gateway/src/config.rs` (doc comment only — no struct field needed).
- **Exit**: `cargo check --workspace` passes; gateway binary compiles with OTEL crates;
  `cargo test -p frf-gateway` passes (OTEL init is gated on env var so tests skip it).

---

### p8-c004 — `SpineSignalService` In-Process Tonic Integration Test

- **Dir**: `openspec/changes/p8-c004-tonic-signal-test/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p8-c001, p8-c002, p8-c005)
- **Description**: Add `crates/frf-gateway/tests/signal_mux.rs` with a test that:
  1. Constructs in-memory mock adapters (no Iggy, no Keto required).
  2. Builds `AppState` with the mock adapters.
  3. Binds a `tonic::transport::Server` to a random localhost port.
  4. Creates a tonic gRPC client pointed at that port.
  5. Opens a bidi `RunAgent` stream; sends an `AgentRunStart` frame; asserts at least one
     `AgentEvent` arrives within a timeout.
  6. Sends an `AgentRunControl { cancel: true }` frame; asserts the stream closes.
  Test is NOT marked `#[ignore]` — it uses only in-memory fakes and must pass in CI without
  any infrastructure. Replace the empty `subscribe_mux.rs` skeleton with a concrete test body.
- **Exit**: `cargo test -p frf-gateway -- signal_mux` passes without any external services.

---

### p8-c005 — `TenantActorRegistry` Sweep + Idle Interval Configurability

- **Dir**: `openspec/changes/p8-c005-registry-config/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes (with p8-c001, p8-c002, p8-c004)
- **Description**: Add two fields to `GatewayConfig` in `crates/frf-gateway/src/config.rs`:
  - `registry_sweep_interval_secs: u64` — env `REGISTRY_SWEEP_INTERVAL_SECS`, default 60.
  - `registry_idle_secs: u64` — env `REGISTRY_IDLE_SECS`, default 300.
  Update `spawn_eviction_task(idle_secs: u64)` signature in `crates/frf-librefang/src/registry.rs`
  to `spawn_eviction_task(idle_secs: u64, sweep_interval_secs: u64)`. Replace the hardcoded
  `Duration::from_secs(60)` with the new parameter. Update the call site in `frf-gateway/src/main.rs`
  to read both values from `GatewayConfig`.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-librefang` passes.

---

### p8-c006 — Criterion Benchmarks for `frf-crdt` and `frf-broker-iggy`

- **Dir**: `openspec/changes/p8-c006-criterion-benchmarks/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: yes
- **Description**: Add `criterion = { version = "0.5", features = ["html_reports"] }` to
  workspace `[dev-dependencies]`. Add `crates/frf-crdt/benches/crdt_merge.rs`:
  - Benchmark `crdt_apply_delta` for 1-op, 100-op, and 1000-op doc histories.
  - Use `criterion::black_box` to prevent dead-code elimination.
  Add `[[bench]] name = "crdt_merge" harness = false` to `crates/frf-crdt/Cargo.toml`.
  Optionally add `crates/frf-broker-iggy/benches/publish_throughput.rs` using an in-memory
  mock broker. Store baseline results in `.criterion/` (gitignore the raw data; commit the
  baseline JSON). Add `cargo bench -p frf-crdt -- --save-baseline main` to the Dagger
  perf stage comment in `dagger/codegen.ts`.
- **Exit**: `cargo bench -p frf-crdt` runs without error and produces output.

---

### p8-c007 — Docker Compose Stack + Dagger Integration Stage

- **Dir**: `openspec/changes/p8-c007-compose-dagger-integration/`
- **Agent**: devops-engineer
- **Deps**: p8-c001 (spine wiring must be complete for Layer 3 tests to exercise it)
- **Description**: Add `compose.yml` at the repo root with services:
  - `gateway` — builds from workspace; exposes 8080 (HTTP), 9090 (gRPC).
  - `iggy-server` — `ghcr.io/iggy-rs/iggy:latest`; persists to named volume.
  - `keto` — `oryd/keto:v0.12`; namespace config from `deploy/keto/`.
  - `oathkeeper` — `oryd/oathkeeper:v0.40`; rules from `deploy/oathkeeper/`.
  - `surrealdb` — `surrealdb/surrealdb:latest`; persists to named volume.
  - `postgres` — `postgres:17-alpine`; logical replication enabled.
  Add `compose.override.yml` (minimal dev): gateway + iggy only, all auth disabled.
  Add `deploy/` directory with minimal config for Keto namespaces and Oathkeeper rules.
  Update `dagger/codegen.ts` to add a Stage 9 (`integration`) that:
  1. Starts the compose stack via `docker compose up -d`.
  2. Waits for gateway `/healthz` with retry.
  3. Runs `playwright test e2e/p7-smoke.spec.ts` with `GATEWAY_URL=http://localhost:8080
     WASM_AVAILABLE=1`.
  4. Tears down with `docker compose down`.
  Stage 9 is only materialized when `ENABLE_INTEGRATION_STAGE=true` env var is set.
- **Exit**: `docker compose up -d` and `docker compose down` work; `compose.yml` is valid YAML;
  Dagger stage compiles and `dagger/codegen.ts` typechecks.

---

### p8-c008 — `frf-policy-cedar` — `ActionPolicyProvider` Port + Cedar Adapter

- **Dir**: `openspec/changes/p8-c008-cedar-policy/`
- **Agent**: rust-reviewer
- **Deps**: none (no other Phase 8 change depends on Cedar)
- **Parallel candidate**: yes (can be done last after all other changes stabilize)
- **Description**: 
  1. Add `ActionPolicyProvider` trait to `crates/frf-ports/src/policy.rs`:
     ```rust
     #[async_trait]
     pub trait ActionPolicyProvider: Send + Sync + 'static {
         async fn is_permitted(&self, principal: &TenantId, action: &str, resource: &str) -> Result<bool, PolicyError>;
     }
     ```
  2. Create `crates/frf-policy-cedar/` with:
     - `Cargo.toml` — `cedar-policy = "4"`, `frf-ports`, `frf-domain`, `thiserror`.
     - `src/lib.rs` — `CedarPolicyEngine` struct wrapping an in-memory `cedar_policy::PolicySet`.
     - `src/policy.cedar` — sample policy: `permit(principal, action == Action::"Publish", resource)`.
     - `src/error.rs` — `PolicyError` using `thiserror`.
  3. Add `frf-policy-cedar` to workspace members in root `Cargo.toml`.
  4. Add `frf-policy-cedar` dep to `frf-gateway/Cargo.toml`.
  5. Add `POLICY_ENGINE` env var to `GatewayConfig` (`cedar` | `none`; default `none`).
  6. When `POLICY_ENGINE=cedar`, wrap publish handler with `action_policy.is_permitted(...)` check.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-policy-cedar` passes;
  `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes.

---

## Dependency Graph

```
p8-c001 (spine wiring)   p8-c002 (instrument audit)   p8-c004 (tonic test)
    │                         │                              │
    │                         ▼                              │ (all parallel)
    │                    p8-c003 (OTEL exporter) ◄───────────┘
    │                         │
    └───────────┬─────────────┘
                ▼
         p8-c007 (Docker Compose + Dagger integration)
                │
         p8-c005 (registry config) ──── parallel, no consumers
         p8-c006 (benchmarks) ────────── parallel, no consumers
         p8-c008 (Cedar) ──────────────── parallel, no consumers
```

p8-c001, p8-c002, p8-c004, p8-c005, p8-c006, p8-c008 are all parallel.
p8-c003 depends on p8-c002 (instruments must be in place).
p8-c007 depends on p8-c001 (spine wiring needed for Layer 3 smoke test).

---

## Quality Gate Protocol

After each Rust change:
1. `cargo check --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
3. `cargo test -p <affected-crate>`

After p8-c007:
- Additionally: `docker compose config` (validates YAML syntax)
- Run `SKIP_INTEGRATION=true pnpm exec playwright test e2e/p7-smoke.spec.ts` to confirm UI layer still passes

After p8-c003:
- `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run -p frf-gateway` must not panic on startup

---

## Phase Exit Criterion

**SATISFIED when**:
- `cargo check --workspace` exits 0.
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0.
- `cargo test -p frf-gateway -- signal_mux` passes without external services.
- `cargo bench -p frf-crdt` runs without error.
- `docker compose config` exits 0.
- `admin-ui/e2e/p7-smoke.spec.ts` Layer 1 tests pass (SKIP_INTEGRATION=true).

Full Layer 3 E2E (Docker Compose + WASM_AVAILABLE=1) is the optional advanced gate, exercised
in CI only when `ENABLE_INTEGRATION_STAGE=true`.
