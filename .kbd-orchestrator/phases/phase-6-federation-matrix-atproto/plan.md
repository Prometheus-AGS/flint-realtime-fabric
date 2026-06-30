# Plan — Phase 6: Federation + Matrix/ATProto Bridge

> Generated: 2026-06-20 · Tool: kbd-plan

---

## Open Decision Resolutions

| Decision | Resolution |
|---|---|
| Tuwunel crate source | Abstract behind `MatrixClient` trait; use `ReqwestMatrixClient` REST stub. Git dep optional/deferred until confirmed. |
| Tranquil / Jetstream client | `tokio-tungstenite` WebSocket consumer against Bluesky Jetstream WS API. No external crate needed. |
| gRPC topology | Dedicated tonic `Server` on port 9090 — separate from Axum on 8080. Avoids HTTP/2 mux complexity. |
| AppState federation dimension | `Vec<Arc<dyn FederationBridge + Send + Sync>>` side-channel field — NOT a 6th generic. Object-safe trait object. |
| `RunAgent` proto shape | Server-streaming only (`request → stream<AgentEvent>`). Bidi deferred. |

---

## Ordered Change List

Changes execute in strict dependency order. No change may begin until the
preceding change's quality gates pass.

### p6-c001 — ADR-002: AgentEventBus Tenant Isolation
- **Dir**: `openspec/changes/p6-c001-adr-agent-bus-tenant/`
- **Agent**: claude (doc-writer)
- **Deps**: none
- **Parallel candidate**: can run in parallel with p6-c002
- **Description**: Write `docs/decisions/adr-002-agent-bus-tenant-isolation.md`
  documenting subscription-scoped tenant isolation, the CLAUDE.md security
  constraint deviation rationale, and subscribe-time Keto as the mitigation.
- **Exit**: `cargo check --workspace` passes; ADR file exists with all required sections.

### p6-c002 — LibreFangBus TenantActorRegistry
- **Dir**: `openspec/changes/p6-c002-librefang-tenant-registry/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: can run in parallel with p6-c001
- **Description**: Replace single global `PublisherActor` with
  `TenantActorRegistry` (DashMap + lazy actor spawn + idle eviction).
  Add `dashmap = "6"` workspace dep.
- **Exit**: `cargo clippy --workspace -- -D warnings -W clippy::pedantic` passes;
  `cargo test -p frf-librefang` passes.

### p6-c003 — frf-bridge-matrix crate
- **Dir**: `openspec/changes/p6-c003-frf-bridge-matrix/`
- **Agent**: rust-reviewer
- **Deps**: p6-c002 (TenantActorRegistry must exist before bridge crates are wired)
- **Description**: Create `crates/frf-bridge-matrix/` implementing
  `FederationBridge for MatrixBridge` via a `MatrixClient` trait abstraction.
  Tuwunel stubbed as `ReqwestMatrixClient`. Unit tests on projection logic.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-bridge-matrix` passes.

### p6-c004 — frf-bridge-atproto crate
- **Dir**: `openspec/changes/p6-c004-frf-bridge-atproto/`
- **Agent**: rust-reviewer
- **Deps**: p6-c002
- **Parallel candidate**: can run in parallel with p6-c003 (same dep level)
- **Description**: Create `crates/frf-bridge-atproto/` implementing
  `FederationBridge for AtProtoBridge` via `tokio-tungstenite` Jetstream WS
  consumer. Write path returns `Err(Unsupported)`.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-bridge-atproto` passes.

### p6-c005 — AgentGrpcService RunAgent handler
- **Dir**: `openspec/changes/p6-c005-agent-grpc-service/`
- **Agent**: rust-reviewer
- **Deps**: p6-c002 (TenantActorRegistry; bus.subscribe routing must work before gRPC handler)
- **Parallel candidate**: can run in parallel with p6-c003 + p6-c004 (no bridge dep)
- **Description**: Add `domain_to_proto` to `frf-agentproto`; create
  `agent_grpc_service.rs` implementing `AgentService.RunAgent`; wire tonic
  `Server` on port 9090 in `main.rs`; add subscribe-time Keto check per ADR-002.
- **Exit**: `cargo clippy --workspace -- -D warnings -W clippy::pedantic` passes;
  gRPC server starts on port 9090 (`GRPC_PORT` env var).

### p6-c006 — Gateway Federation Wiring + Admin UI Debt
- **Dir**: `openspec/changes/p6-c006-gateway-federation-wiring/`
- **Agent**: rust-reviewer + typescript-reviewer
- **Deps**: p6-c003, p6-c004, p6-c005 (all bridge crates + gRPC handler must exist)
- **Description**: Wire bridges into `AppState.federation_bridges`; add
  background ingest tasks in `main.rs`; add `AppStateArc` type alias; remove 3×
  `type_complexity` allows; fix admin-ui demo token and ring-buffer export.
- **Exit**: `cargo clippy -- -D warnings -W clippy::pedantic` 0 `type_complexity`;
  `pnpm typecheck` passes; no hardcoded tokens.

### p6-c007 — Phase 6 E2E Smoke Test
- **Dir**: `openspec/changes/p6-c007-e2e-federation-smoke/`
- **Agent**: e2e-runner
- **Deps**: p6-c006 (full gateway + admin-ui wiring must be complete)
- **Description**: Write `admin-ui/e2e/phase6-smoke.spec.ts` (3-layer pattern).
  Add dev-only `POST /dev/inject-federation-event` gateway route. Layer 1 tests
  pass with `SKIP_INTEGRATION=true`. Phase 6 exit criterion satisfied.
- **Exit**: All Layer 1 tests pass; Layer 2+3 skip cleanly; `pnpm typecheck` passes.

---

## Dependency Graph

```
p6-c001 (ADR)          p6-c002 (TenantActorRegistry)
    │                       │
    │              ┌────────┴──────────────┐
    │              ▼                       ▼
    │          p6-c003 (Matrix)       p6-c004 (ATProto)
    │              │                       │
    │              └──────────┬────────────┘
    │                         │
    │                    p6-c005 (RunAgent gRPC)
    │                         │
    │                    p6-c006 (Wiring + Debt)
    │                         │
    └─────────────────────────▼
                        p6-c007 (E2E)
```

p6-c001 and p6-c002 run in parallel (no mutual deps).
p6-c003 and p6-c004 run in parallel (same dep level: p6-c002).
p6-c005 runs in parallel with p6-c003 + p6-c004 (dep: p6-c002 only).
p6-c006 is the sync point requiring all three of p6-c003, p6-c004, p6-c005.
p6-c007 is the final gate change.

---

## Quality Gate Protocol

After each change, the executor must run (in order):

1. `cargo check --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
3. Change-specific test command (e.g., `cargo test -p frf-bridge-matrix`)

A change is DONE only when all three pass with 0 errors/warnings.

After p6-c006:
- Additionally run `cd admin-ui && pnpm typecheck && pnpm build`

After p6-c007:
- Additionally run `cd admin-ui && SKIP_INTEGRATION=true pnpm exec playwright test e2e/phase6-smoke.spec.ts`

---

## Carry-Forward Debt Resolution Tracking

| Debt Item | Resolution Change |
|---|---|
| Keto ADR for agent bus | p6-c001 |
| Single actor tree, no sharding | p6-c002 |
| `#[allow(clippy::type_complexity)]` × 3 | p6-c006 |
| Demo WS token in `agentWebSocket.ts` | p6-c006 |
| E2E ring-buffer test broken | p6-c006 + p6-c007 |

All 5 Phase 5 debt items are addressed in Phase 6.

---

## Phase Exit Criterion

**SATISFIED when**: `admin-ui/e2e/phase6-smoke.spec.ts` Layer 1 tests all pass
with `SKIP_INTEGRATION=true`, AND `cargo clippy --workspace --all-targets -- -D
warnings -W clippy::pedantic` exits 0, AND `pnpm typecheck` in `admin-ui/`
exits 0.

The full exit criterion (Matrix event in admin-ui stream) is exercised in Layer
3 which requires a full stack. For CI without a live gateway, the exit criterion
is met when Layer 1 passes and the infrastructure code compiles clean.
