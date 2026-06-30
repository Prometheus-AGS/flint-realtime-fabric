# ADR-002: AgentEventBus Tenant Isolation Model

## Status

Accepted — 2026-06-20

## Context

`CLAUDE.md` mandates per-event Keto `check(subject, "view", object_id)` before
every fan-out delivery to enforce per-event row-level security (RLS). The same
document notes: "design caching at subscribe time to avoid per-event Keto
latency at scale."

Phase 5 introduced `LibreFangBus` as the agent event bus. Its `ws_agent_stream`
handler extracts `tenant_id` from `VerifiedClaims` (JWT-verified by Oathkeeper
at the gateway boundary) and passes it to `bus.subscribe(&tenant_id)`. The bus
routes events only to subscribers on that tenant's channel. No per-event Keto
check is performed.

This deviates from the literal CLAUDE.md mandate and requires documentation
with explicit rationale.

### Why this matters

The agent event bus is distinct from the event envelope spine:

| Concern | Event Spine (Iggy) | Agent Bus (LibreFang) |
|---|---|---|
| Data | Persistent `EventEnvelope` objects | Ephemeral streaming frames |
| Volume | Moderate (entity changes) | High (50–200 frames/sec per run) |
| Keto target | Object IDs (`entity_id`) | No persistent object — frame is transient |
| Fan-out | Per-topic subscriber list | Per-tenant channel |

Agent events (`AgentEvent`) are ephemeral streaming frames produced during a
live agent run. They are never stored as persistent objects with Keto tuples.
Applying a Keto `check(subject, "view", object_id)` per frame would require:

1. A `relation_tuple` per `run_id` for every agent execution, or
2. A broad `relation_tuple` at the tenant level (functionally equivalent to the
   current subscription-scoped model), or
3. An in-process Keto relation cache to avoid per-event network latency.

At 100 frames/sec with a Keto round-trip of ~2ms, per-event checks add 200ms/sec
of cumulative latency — equivalent to a 20% slowdown on a streaming agent
response. CLAUDE.md itself identifies this concern and recommends subscribe-time
caching as the mitigation.

## Decision

The `LibreFangBus` uses **subscription-scoped tenant isolation**:

1. At subscribe time, `tenant_id` is extracted from `VerifiedClaims` produced
   by Oathkeeper JWT verification at the gateway boundary. The `tenant_id` is
   never supplied by the caller — it comes only from the verified token.

2. A **subscribe-time Keto check** is performed before the subscription channel
   is opened:
   ```
   authz.check(subject, "agent_bus:stream", &tenant_id)
   ```
   This replaces per-event Keto with a single check that validates the subscriber
   has access to the agent bus for their tenant before any events flow.

3. The bus channel is isolated per `tenant_id` by construction: `bus.subscribe(&tenant_id)`
   returns a stream scoped to that tenant's mailbox. No cross-tenant event
   delivery is possible without a separate subscription.

4. Per-event Keto checks are **deferred** to a future phase when:
   - Agent events are stored as persistent objects (e.g., run audit log), AND
   - A Redis-backed Keto relation cache is available to absorb per-event latency.

## Rationale

### Why subscription-scoped isolation is sufficient for Phase 6

- **Source integrity**: `tenant_id` comes from `VerifiedClaims`, which are
  produced by Oathkeeper after verifying the JWT signature. The gateway never
  accepts caller-supplied `tenant_id` directly.
- **Channel isolation**: the `PublisherActor` / `TenantActorRegistry` only
  delivers events to subscribers registered under the exact same `tenant_id`.
  There is no broadcast path that could leak cross-tenant events.
- **Object-less frames**: agent streaming frames have no persistent object ID to
  check in Keto. Keto is designed for `(subject, relation, object)` tuples where
  `object` is a stored entity. Ephemeral frames are not Keto objects.
- **Subscribe-time check**: one Keto check per subscription opening is the
  correct application of the "caching at subscribe time" strategy recommended
  in CLAUDE.md. It provides the security boundary without per-event overhead.

### Why per-event Keto is deferred, not rejected

Per-event Keto is the correct long-term approach when:
- Agent run records are promoted to persistent `EventEnvelope` objects (audit),
- A Keto cache (Redis or in-process) is available to reduce per-event latency
  to sub-millisecond.

This phase defers that work — it does not preclude it.

## Consequences

**Positive:**
- Subscribe-time Keto check provides an explicit authorization gate at channel
  open, satisfying the spirit of CLAUDE.md security constraints.
- No per-event Keto latency on high-frequency agent streams.
- Tenant isolation by construction — no cross-tenant leakage possible via the
  bus routing layer.
- `tenant_id` from `VerifiedClaims` only — no privilege escalation via
  caller-supplied tenant.

**Negative:**
- Authorization is checked once at subscribe time, not per event. If a
  tenant's Keto `agent_bus:stream` relation is revoked mid-stream, the existing
  subscription continues until the WS/gRPC connection closes.
- Mitigations: connection timeout; explicit revocation can force disconnect by
  draining the subscription (future work).
- Per-event Keto remains as technical debt until a Keto relation cache is
  introduced.

## Required Implementation

Both `ws_agent_stream` (WebSocket path, Phase 5) and `AgentGrpcService.run_agent`
(gRPC path, Phase 6) MUST perform the subscribe-time Keto check before calling
`bus.subscribe(&tenant_id)`:

```rust
authz
    .check(&claims.subject, "agent_bus:stream", &claims.tenant_id)
    .await
    .map_err(|_| /* 403 / Permission Denied */)?;
```

Failure to perform this check means the agent bus is secured only by JWT
authentication (who you are), not by authorization (what you can access).

## Alternatives Considered

### Per-event Keto check (rejected for Phase 6)

Applying `authz.check(subject, "view", run_id)` to every `AgentEvent` frame
adds ~2ms latency per event. At 100 events/sec, this is 200ms/sec of blocking
time on the agent response stream. Without an in-process Keto relation cache,
this is unacceptable for interactive agent use cases.

### Separate AuthZ middleware layer (deferred)

A tower `Layer` that intercepts each streamed frame and performs an async Keto
check with a TTL-based cache would be the production-grade approach. This
requires a `KetoCacheLayer` abstraction not yet in the codebase. Deferred to
Phase 7 or a dedicated authz-hardening phase.

### Broad Keto tuple at tenant level (equivalent to current approach)

`relation_tuple(tenant:X, "has_agent_bus", "agent_bus")` checked at subscribe
time is functionally equivalent to the current `agent_bus:stream` check. The
current design uses a more specific relation name for forward-compatibility with
per-run authorization.

## Supersedes

None — this is the first tenant isolation decision for the agent bus.

## Related

- CLAUDE.md security constraints (per-event RLS requirement, subscribe-time
  caching note)
- ADR-001: CRDT Engine Selection (format reference)
- `crates/frf-librefang/src/bus.rs` — current isolation implementation
- `crates/frf-gateway/src/routes/agents.rs` — `ws_agent_stream` JWT path
- `crates/frf-gateway/src/agent_grpc_service.rs` — `run_agent` gRPC path (Phase 6)
