# Assessment — Phase 6: Federation + Matrix/ATProto Bridge

> Generated: 2026-06-20 · Tool: kbd-assess

---

## Codebase Scan Summary

### Workspace Inventory (17 crates)

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
| `frf-agentproto` | EXISTS (Phase 5) |
| `frf-librefang` | EXISTS (Phase 5) |
| `frf-crdt` | EXISTS |
| `frf-store-redb` | EXISTS |
| `frf-store-surreal` | EXISTS |
| `frf-postgres-cdc` | EXISTS |
| `frf-media-livekit` | EXISTS |
| `frf-ffi` | EXISTS |
| `frf-wasm` | EXISTS |
| **`frf-bridge-matrix`** | **MISSING** — must create |
| **`frf-bridge-atproto`** | **MISSING** — must create |

---

## Gap Analysis by Goal

### Goal 1: `frf-bridge-matrix` — Tuwunel Matrix homeserver bridge

**Status: MISSING crate — port trait fully ready**

Existing foundations:
- `crates/frf-ports/src/federation.rs` — **EXISTS and complete**. Defines:
  - `FederationProtocol` enum (`Matrix`, `AtProto`) — `#[non_exhaustive]`
  - `FederatedEvent { protocol, source, envelope }` struct
  - `FederationStream` type alias (`Pin<Box<dyn Stream<...>>>`)
  - `FederationBridge` trait with `send()` and `subscribe()` async methods
- `frf-ports/src/lib.rs` — re-exports `FederatedEvent`, `FederationBridge`, `FederationProtocol`

**Gaps:**
- No `frf-bridge-matrix` crate exists.
- No Tuwunel dependency in workspace `Cargo.toml` — must be resolved (git dep vs vendored; see Open Decisions below).
- No Matrix room → `EventEnvelope` projection logic.
- No `AppState` dimension for federation bridge adapters (AppState currently has 5 generics: `L, A, I, M, B`). Adding federation may require a 6th generic `F: FederationBridge` or a `Vec<Box<dyn FederationBridge>>` side-channel in `AppState`.

**Work needed:**
1. Resolve Tuwunel crate source.
2. Create `crates/frf-bridge-matrix/` implementing `FederationBridge` for `FederationProtocol::Matrix`.
3. Decide `AppState` federation wiring strategy (6th generic vs `Arc<dyn FederationBridge>`).
4. Add federation ingest background task in `main.rs` (analogous to CDC consumer task).

---

### Goal 2: `frf-bridge-atproto` — Tranquil firehose projection

**Status: MISSING crate — port trait fully ready**

Existing foundations:
- Same `FederationBridge` port trait as Goal 1.
- No ATProto-specific types or proto definitions.

**Gaps:**
- No `frf-bridge-atproto` crate exists.
- No Tranquil / Jetstream Rust crate dependency. Must resolve whether a `tokio`-native Jetstream consumer exists or if `atrium-api` + manual SSE parsing is needed.
- ATProto Jetstream events differ structurally from Matrix room events — the projection to `EventEnvelope` needs its own mapping.

**Work needed:**
1. Resolve Tranquil / Jetstream crate availability.
2. Create `crates/frf-bridge-atproto/` implementing `FederationBridge` for `FederationProtocol::AtProto`.
3. Implement Jetstream SSE consumer (or Tranquil firehose consumer) producing `FederatedEvent` stream.

---

### Goal 3: `AgentService.RunAgent` gRPC streaming

**Status: NOT STARTED — proto compiled, server trait generated, no handler**

Existing foundations:
- `proto/flint/v1/agent.proto` — **EXISTS and compiled**:
  ```proto
  service AgentService {
    rpc RunAgent(AgentRunRequest) returns (stream AgentEvent);
  }
  ```
- `frf-proto` uses `tonic-build` — `agent_service_server::AgentService` trait IS generated (server-streaming, not bidi; proto definition is `request → stream<response>`).
- `frf-agentproto` — `ContentBlock` typed union, `AgentProtoError`, `domain_from_proto` converter — all present.
- `frf-librefang` — `LibreFangBus.publish()` and `subscribe()` are fully operational.
- `frf-gateway/src/grpc_service.rs` — only implements `SpineService` (envelope pub/sub). No `AgentService` handler.

**Gaps:**
- No `AgentGrpcService` struct in `frf-gateway`.
- No gRPC `RunAgent` handler wired into the tonic server builder in `main.rs`.
- `AppState` carries the `agent_bus: Arc<B>` and `identity: Arc<I>` already — the handler simply needs to call `bus.subscribe(tenant_id)` after JWT verification and stream the results to the gRPC client as `AgentEvent` proto messages.
- No conversion from `frf_domain::AgentEvent` → `fv1::AgentEvent` (inverse of `domain_from_proto`); must add `domain_to_proto` in `frf-agentproto/src/convert.rs`.
- `main.rs` does not start a tonic `Server` — it only starts Axum. tonic 0.14 supports serving gRPC over HTTP/2 via Axum's `Router::route_service` or via a standalone tonic server on a second port. The current stack has no gRPC server wired at runtime.

**Critical open question:** Does Phase 6 start a dedicated tonic gRPC server on port 9090 (separate from Axum on 8080), or use Axum's `axum::routing::Router::merge` with `tonic_web` / Connect protocol? The existing `signal_service.rs` and `sync_grpc_service.rs` exist but are never added to the running server.

**Work needed:**
1. Add `domain_to_proto(domain: AgentEvent) -> fv1::AgentEvent` to `frf-agentproto/src/convert.rs`.
2. Create `crates/frf-gateway/src/agent_grpc_service.rs` implementing `AgentService` trait.
3. Wire a tonic gRPC server (either second port or via `tower`/`hyper` HTTP/2 layer under Axum).
4. Add JWT-gate at the gRPC boundary (analogous to `signal_service.rs`).

---

### Goal 4: AgentEventBus tenant isolation ADR

**Status: NOT STARTED — ADR-001 (CRDT engine) exists as template**

Existing foundations:
- `docs/decisions/adr-001-crdt-engine.md` — EXISTS; provides format template.
- Current isolation behavior: `ws_agent_stream` extracts `tenant_id` from `VerifiedClaims` (JWT-verified by `IdentityVerifier`) and passes it to `bus.subscribe(&tenant_id)`. This is the only security boundary for the agent bus — the bus itself does no additional Keto check.

**Gap:** No ADR documents this decision. The bus is subscription-scoped, not event-scoped; a Keto `check(subject, "view", object_id)` per event (as specified for the event spine in CLAUDE.md) is absent from the agent bus path. This deviation from the CLAUDE.md per-event RLS specification must be documented with explicit rationale.

**Work needed:**
- Write `docs/decisions/adr-002-agent-bus-tenant-isolation.md` documenting:
  - Current: `tenant_id` filter on `subscribe()` (bus-level tenant scoping)
  - Why no per-event Keto: agent events are ephemeral streaming frames, not persistent entities; Keto is designed for object-level checks; applying it per-event on high-frequency agent streams would require caching at subscribe time (as CLAUDE.md notes)
  - Mitigation: JWT verification is enforced at the gateway WS boundary; `tenant_id` is extracted from verified claims only

---

### Goal 5: LibreFangBus actor sharding (`TenantActorRegistry`)

**Status: NOT STARTED — single global PublisherActor confirmed**

Current implementation:
```rust
// bus.rs: LibreFangBus holds ONE ActorRef<PublisherMessage>
pub struct LibreFangBus {
    publisher: ActorRef<PublisherMessage>,
}
```

The `PublisherActor` holds all tenant subscriber maps in a single actor mailbox. Under high concurrency, all tenant events queue on a single actor, creating a throughput bottleneck.

**Gap:** No `TenantActorRegistry`. The `PublisherActor` is not sharded. No per-tenant mailbox capacity limits.

**Work needed:**
1. Add `TenantActorRegistry` to `frf-librefang/src/`: a `DashMap<String, ActorRef<PublisherMessage>>` that lazily creates one `PublisherActor` per tenant.
2. Wire idle-timeout eviction (drop actor if no subscribers for N seconds).
3. Update `LibreFangBus` to route via `TenantActorRegistry` instead of the single global actor.
4. Confirm `ractor 0.15` supervisor subtree API supports dynamic child spawning (it does — `Actor::spawn_linked` or `SupervisionStrategy` on the root actor).

---

### Goal 6: Phase exit criterion — E2E test

**Status: NOT STARTED — depends on Goals 1–5**

Exit criterion: A Matrix room event ingested via Tuwunel appears in the admin-ui entity stream table, confirmed by `admin-ui/e2e/phase6-smoke.spec.ts`.

**Gap:** No `phase6-smoke.spec.ts`. No Matrix room fixture. Federation bridge not yet in codebase.

---

## Phase 5 Carry-Forward Debt Status

| Item | Status | Action |
|---|---|---|
| Keto ADR for agent bus | NOT DONE | Goal 4 in this phase |
| Single actor tree, no sharding | NOT DONE | Goal 5 in this phase |
| `#[allow(clippy::type_complexity)]` in 3 route handlers | NOT DONE | Address in p6-c003 or p6-c004 when `AppState` changes |
| Demo WS token in `agentWebSocket.ts` | NOT DONE | Address in p6-c005 (admin-ui) alongside federation UI |
| E2E ring-buffer test `window.__agentEventStore` | NOT DONE | Fix in p6-c006 E2E pass |

---

## Open Decisions (resolve before plan kickoff)

| Decision | Impact | Recommendation |
|---|---|---|
| **Tuwunel crate source** — git dep, crates.io, or vendored? | Blocks `frf-bridge-matrix` `Cargo.toml` | Use `{ git = "https://github.com/GQAdonis/tuwunel" }` if available; otherwise vendor as `crates/frf-bridge-matrix/vendor/` |
| **Tranquil / Jetstream Rust client** — crate or manual? | Blocks `frf-bridge-atproto` | If no crate: implement SSE consumer against Bluesky Jetstream HTTP API using `reqwest` + `tokio` + `eventsource-stream` |
| **gRPC server topology** — second port or Axum-integrated? | Blocks Goal 3 | **Recommendation: second port (9090)** for gRPC; `tonic::Server` alongside Axum; avoids HTTP/2 vs HTTP/1.1 mux complexity on the same port |
| **`AppState` federation dimension** — 6th generic or `Arc<dyn FederationBridge>`? | Blocks Goals 1–2 gateway wiring | **Recommendation: `Arc<dyn FederationBridge + Send + Sync>`** in a `Vec` — federation bridges are background consumers, not per-request state; avoid 6th generic |
| **`RunAgent` proto shape** — server-stream vs bidi? | Confirmed from proto | Proto is `request → stream<response>` (server-streaming). Phase 6 implements server-streaming only; bidi deferred. |

---

## Dependency Order

```
docs/decisions/adr-002-agent-bus-tenant-isolation.md   (no code deps)
   ↓ parallel with
TenantActorRegistry in frf-librefang                   (standalone)
   ↓ resolve open decisions, then:
frf-bridge-matrix   (implements FederationBridge — no AppState change needed for background task)
   ↓ parallel with
frf-bridge-atproto  (same pattern)
   ↓ both bridge crates done:
frf-gateway agent_grpc_service.rs  (AgentService.RunAgent + gRPC server wiring)
   ↓
Admin UI federation event display + demo token fix
   ↓
Phase 6 E2E smoke test (phase6-smoke.spec.ts)
```

---

## Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| Tuwunel not published / incompatible API | HIGH | Vendor as in-tree module; stub the homeserver client |
| Tranquil Jetstream crate does not exist | MEDIUM | Use `reqwest` + `eventsource-stream` against Bluesky public Jetstream directly |
| gRPC server on second port requires TLS/H2 config | MEDIUM | Use `tonic::Server::accept_http1(true)` for dev; document production TLS requirement |
| `AppState` 6th generic creates combinatorial where-clause explosion | MEDIUM | Use `Arc<dyn FederationBridge>` (object-safe) instead of generic |
| ractor 0.15 dynamic child spawn under load causes mailbox overrun | LOW | Add per-tenant `mailbox_capacity` in `Actor::spawn` options; test under concurrent load |
| Phase 5 ring-buffer E2E test broken (no `window.__agentEventStore`) | LOW | Fix in Phase 6 E2E pass; export store via `window.__frf_dev = { agentEventStore }` in dev mode |

---

## Assessment Verdict

Phase 6 is **buildable without blocking ambiguities** once the two external dependency decisions (Tuwunel, Tranquil/Jetstream) are resolved. The `FederationBridge` port trait, `FederationProtocol` enum, and `FederatedEvent` type are fully defined and re-exported from `frf-ports`. The agent proto, bus, and gateway WS infrastructure from Phase 5 provide the exact foundation needed for `RunAgent` gRPC streaming. The CRDT ADR template in `docs/decisions/adr-001-crdt-engine.md` serves as the format for the new ADR-002.

**Recommended first action for plan phase:** Resolve the Tuwunel git URL and Tranquil/Jetstream client strategy before drafting `Cargo.toml` files for the two bridge crates, and confirm the gRPC server topology (second port recommended).

**Planned change count:** 7 changes (ADR + actor sharding + matrix bridge + atproto bridge + RunAgent gRPC + admin-UI debt + E2E).
