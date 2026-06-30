# Assessment — Phase 5: Agent Protocols + BossFang (LibreFang fork)

> Generated: 2026-06-20 · Tool: kbd-assess

---

## Codebase Scan Summary

### Workspace Inventory (15 crates)

| Crate | Status |
|---|---|
| `frf-domain` | EXISTS |
| `frf-ports` | EXISTS |
| `frf-app` | EXISTS |
| `frf-proto` | EXISTS |
| `frf-gateway` | EXISTS |
| `frf-broker-iggy` | EXISTS |
| `frf-authz-keto` | EXISTS |
| `frf-identity-ory` | EXISTS |
| `frf-crdt` | EXISTS |
| `frf-store-redb` | EXISTS |
| `frf-store-surreal` | EXISTS |
| `frf-postgres-cdc` | EXISTS |
| `frf-media-livekit` | EXISTS |
| `frf-ffi` | EXISTS |
| `frf-wasm` | EXISTS |
| **`frf-agentproto`** | **MISSING** — must create |
| **`frf-librefang`** | **MISSING** — must create |

---

## Gap Analysis by Goal

### Goal 1: Implement `frf-agentproto` crate

**Status: MISSING crate — proto types already exist**

Existing foundations:
- `proto/flint/v1/agent.proto` — **EXISTS and compiled**. Full `AgentProtocol` enum (AG_UI, A2A, A2UI), `AgentEventKind` enum (7 kinds: RUN_START, RUN_END, TEXT_DELTA, TOOL_CALL, TOOL_RESULT, STATE_SNAPSHOT, ERROR), `AgentEvent` message, `AgentService { rpc RunAgent(AgentRunRequest) returns (stream AgentEvent) }`.
- `frf-domain/src/agent.rs` — **EXISTS**. `AgentProtocol`, `AgentEventKind`, `AgentEvent` Rust domain types with serde. Matches proto structure.
- `frf-proto/build.rs` — `agent.proto` IS compiled via tonic-build. `AgentService` tonic server trait will be generated.

**Gaps:**
- No `frf-agentproto` crate exists. This crate must house the AG-UI / A2A / A2UI schema validation layer, `ContentBlock` types, and the mapping between proto-generated types and domain types.
- `ContentBlock` (text, tool call, tool result, state snapshot payloads) not yet defined anywhere as typed structs — `content` is a raw `serde_json::Value` in the domain.
- No `AgentEventBus` port trait in `frf-ports` — nothing to publish/receive agent events through a port seam.

**Work needed:** Create `crates/frf-agentproto/` with typed `ContentBlock` variants, conversions from `frf-proto` generated types, and add `AgentEventBus` trait to `frf-ports`.

---

### Goal 2: Implement `frf-librefang` crate (BossFang actor bus)

**Status: MISSING crate — ractor workspace dep already pinned**

Existing foundations:
- `Cargo.toml` workspace dep: `ractor = "0.15"` **EXISTS**.

**Gaps:**
- No `frf-librefang` crate exists. This crate wraps/vendors BossFang (GQAdonis fork of LibreFang), a ractor-based publish/consume actor framework.
- BossFang fork URL / crates.io path is **not yet confirmed** — this is an open prerequisite decision (git dep vs vendored).
- No actor-bus port trait (`AgentEventBus`) in `frf-ports` for the crate to implement.
- No supervisor actor tree wiring anywhere in `frf-gateway`.

**Open decision:** Single BossFang actor tree per workspace OR one per tenant. This affects whether `AppState` holds one `Arc<ActorBus>` or a `HashMap<TenantId, Arc<ActorBus>>`.

**Work needed:**
1. Resolve BossFang fork source (git dep or crates.io).
2. Create `crates/frf-librefang/` with publisher actor, subscriber actor, and supervisor tree.
3. Add `AgentEventBus` port trait to `frf-ports`.
4. Implement `AgentEventBus` in `frf-librefang` using ractor actors.

---

### Goal 3: Wire `frf-librefang` into `frf-gateway`

**Status: NOT STARTED — AppState has no actor bus dimension**

Existing state:
```rust
pub struct AppState<L, A, I, M> {
    pub subscribe_pipeline: Arc<SubscribePipeline<L, A, I>>,
    pub publish_usecase: Arc<PublishUseCase<L, A, I>>,
    pub media_signaler: Arc<M>,
    pub config: Arc<GatewayConfig>,
}
```

**Gaps:**
- `AppState<L, A, I, M>` has 4 generics — no actor bus dimension (no `B: AgentEventBus`).
- `frf-gateway/src/main.rs` wires `IggyBroker` + `Arc<broker>` directly. No actor indirection for agent events.
- `frf-app` use-cases call `self.broker.publish()` / `self.broker.subscribe()` directly via `LogBroker` — agent events would need to flow through the `AgentEventBus`, not the log broker.
- The `AgentService` gRPC impl for `RunAgent` does not yet exist as a concrete handler in `frf-gateway`.

**Work needed:**
- Add `B: AgentEventBus` generic to `AppState` and `build_router`.
- Wire `LibreFangBus` (BossFang actor supervisor) as the `B` implementation in `main.rs`.
- Implement `AgentServiceImpl` in `frf-gateway/src/` that streams `AgentEvent` from the bus to gRPC clients.
- Add WS multiplexer route for agent event streams alongside the existing `/ws/v1/subscribe`.

---

### Goal 4: Admin UI agent activity panel

**Status: NOT STARTED — no `agents` feature directory**

Existing state:
- `admin-ui/src/features/entities/` — EXISTS
- `admin-ui/src/features/signaling/` — EXISTS
- `admin-ui/src/features/agents/` — **MISSING**

**Work needed:**
- Create `admin-ui/src/features/agents/` with:
  - `AgentActivityPanel` component — live event stream table/feed
  - `useAgentEventStream` hook — WebSocket subscriber for AG-UI events
  - `AgentEventStore` (Zustand) — holds the event ring buffer
  - Types: `AgentEvent`, `ContentBlock` (TypeScript mirrors of proto types)
- Wire the panel into `admin-ui/src/App.tsx` or a dedicated agents route.

---

### Goal 5: Phase exit criterion

**Status: NOT STARTED — depends on Goals 1–4**

Exit criterion: An AG-UI agent event flows through BossFang → browser WebSocket consumer.

End-to-end path required:
1. Client (or test fixture) calls `AgentService.RunAgent` via gRPC
2. Gateway `AgentServiceImpl` publishes `AgentEvent` to `LibreFangBus`
3. `LibreFangBus` subscriber actor fans out to registered WS consumers
4. Browser (admin-ui) receives the event over `/ws/v1/agents` (or multiplexed channel)
5. `AgentActivityPanel` renders the event

E2E Playwright test needed alongside the existing `phase4-smoke.spec.ts` pattern.

---

## OpenSpec Changes Needed

No p5 changes exist yet. The following new OpenSpec changes are needed:

| ID | Title |
|---|---|
| `p5-c001-agent-event-bus-port` | Add `AgentEventBus` port trait to `frf-ports` |
| `p5-c002-frf-agentproto` | New crate: `frf-agentproto` with `ContentBlock` types |
| `p5-c003-frf-librefang` | New crate: `frf-librefang` (BossFang actor bus adapter) |
| `p5-c004-gateway-agent-service` | `AgentServiceImpl` + `AppState` actor bus wiring |
| `p5-c005-admin-ui-agents` | Admin UI `agents` feature with activity panel |
| `p5-c006-e2e-agent-smoke` | Playwright E2E for AG-UI event end-to-end |

---

## Prerequisite Decisions (must resolve before plan)

| Decision | Impact |
|---|---|
| BossFang fork source (git dep vs crates.io) | Blocks `frf-librefang` Cargo.toml |
| Single actor tree vs per-tenant | Affects `AppState` shape and memory model |
| AG-UI spec version from upstream | Affects `ContentBlock` schema in `frf-agentproto` |
| ractor 0.15 confirmed current? | Check crates.io before plan; already pinned |

---

## Dependency Order

```
frf-ports (AgentEventBus trait)
  ↓
frf-agentproto (ContentBlock + proto conversions)
  ↓
frf-librefang (BossFang adapter implementing AgentEventBus)
  ↓
frf-gateway (AppState<..., B> + AgentServiceImpl)
  ↓
admin-ui agents feature
  ↓
E2E smoke test
```

All 6 planned changes are strictly sequential — no parallelism available.

---

## Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| BossFang fork not published to crates.io | HIGH | Use `{ git = "..." }` dep; document in ADR |
| ractor 0.15 API churn before plan | MEDIUM | Pin exact version; check changelog |
| Per-tenant actor tree causes memory pressure | MEDIUM | Start with single tree; add sharding later |
| AG-UI spec not stable enough for ContentBlock | LOW | Model `content` as tagged union; Unknown variant for forward compat |
| AppState generic explosion (5 generics) | LOW | Extract to a struct of Arc<dyn Trait> if arity grows further |

---

## Assessment Verdict

Phase 5 is **buildable with no blocking ambiguities** once the BossFang fork URL is confirmed.
The proto, domain types, and ractor workspace dep are already in place.
The dependency order is clear. The 6 changes map directly to the 5 stated goals + exit criterion.

**Recommended first action for plan phase:** resolve the BossFang git URL before drafting the Cargo.toml for `frf-librefang`.
