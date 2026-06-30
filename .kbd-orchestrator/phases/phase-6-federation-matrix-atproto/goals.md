# Goals — Phase 6: Federation + Matrix/ATProto Bridge

> Seeded from Phase 5 reflection · 2026-06-20

## Context

Phase 5 completed the AG-UI / A2A / A2UI agent protocol tier and wired the
`LibreFangBus` actor bus into the gateway. Phase 6 now opens the federation
layer: two bridge crates connect the FRF spine to external federated protocols
(Matrix via Tuwunel, ATProto via Tranquil), and the `AgentService` gRPC
streaming endpoint receives its full bidi implementation.

Phase 5 introduced three debt items that Phase 6 must address before shipping
federation traffic:

1. **AgentEventBus ADR** — tenant isolation for the agent bus is undocumented;
   Keto governs visibility but no ADR records this decision.
2. **Actor tree sharding** — current `LibreFangBus` is a single workspace-level
   supervisor tree with no per-tenant mailbox limits.
3. **`AgentService.RunAgent` gRPC** — the streaming bidi endpoint exists in
   proto but has no concrete handler in `frf-gateway`.

## Goals

1. **`frf-bridge-matrix`** — Tuwunel projection: subscribe to Matrix room
   events via Tuwunel homeserver bridge; project them onto the FRF spine as
   `EventEnvelope` frames; implement the `MatrixBridge` adapter behind a port trait.

2. **`frf-bridge-atproto`** — Tranquil firehose projection: consume ATProto
   Jetstream / firehose events; project onto FRF spine; implement the
   `AtProtoBridge` adapter behind a port trait.

3. **`AgentService.RunAgent` gRPC streaming** — implement the bidi streaming
   `RunAgent` handler in `frf-gateway` using `LibreFangBus`; wire `frf-agentproto`
   ContentBlock encoding/decoding into the RPC path.

4. **AgentEventBus tenant isolation ADR** — write
   `docs/decisions/adr-002-agent-bus-tenant-isolation.md` documenting that the
   agent bus is subscription-scoped by `tenant_id` extracted from `VerifiedClaims`,
   and that Keto governs per-event visibility (not the bus itself).

5. **LibreFangBus actor sharding** — add per-tenant mailbox capacity cap to
   `LibreFangBus`; implement a `TenantActorRegistry` that creates one supervisor
   subtree per active tenant (lazy init, idle timeout eviction).

6. **Phase exit criterion** — a Matrix room event ingested via Tuwunel appears
   in the admin-ui entity stream table, confirmed by a Playwright E2E test
   (`phase6-smoke.spec.ts`).

## Pre-conditions (resolve before plan kickoff)

- Confirm Tuwunel crate availability: git dep, crates.io, or vendored?
  Check `https://github.com/GQAdonis/tuwunel` for FRF-compatible fork.
- Confirm Tranquil crate / Jetstream API stability: does a Rust Jetstream
  consumer crate exist or must we use `atrium-api`?
- Decide `AgentService.RunAgent` streaming shape: full bidi vs server-stream
  only for Phase 6 (client-streaming can come later).
- Confirm `ractor 0.15` supervisor subtree API supports dynamic child creation
  (needed for `TenantActorRegistry`).

## Carry-Forward Debt from Phase 5

| Debt Item | Work in Phase 6 |
|---|---|
| Keto ADR for agent bus missing | Write `adr-002-agent-bus-tenant-isolation.md` |
| Single actor tree, no sharding | `TenantActorRegistry` with per-tenant subtrees |
| `#[allow(clippy::type_complexity)]` in 3 route handlers | Extract `AppStateArc<L,A,I,M,B>` type alias in `frf-gateway/src/lib.rs` |
| Demo WS token in `agentWebSocket.ts` | Replace with real Oathkeeper token flow |
| E2E ring-buffer test needs `window.__agentEventStore` | Export store in dev mode; update phase5 smoke test |
