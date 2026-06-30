# p6-c001 — ADR-002: AgentEventBus Tenant Isolation

## Summary

Write `docs/decisions/adr-002-agent-bus-tenant-isolation.md` documenting the
tenant isolation model for the agent event bus introduced in Phase 5.

## Motivation

CLAUDE.md mandates per-event Keto `check(subject, "view", object_id)` before
every fan-out delivery. The Phase 5 `LibreFangBus` uses subscription-scoped
`tenant_id` filtering instead (bus-level isolation, not event-level Keto). This
architectural deviation must be documented with explicit rationale to satisfy the
security constraint while explaining the deliberate trade-off.

## Design

The ADR records:

1. **Decision**: agent bus uses subscription-time `tenant_id` isolation, not
   per-event Keto checks.
2. **Context**: agent events are high-frequency ephemeral streaming frames (50–200
   events/sec per agent run). Per-event Keto latency (~1–5ms) would add 50–1000ms
   of cumulative delay per second of agent streaming. CLAUDE.md itself notes
   "design caching at subscribe time to avoid per-event Keto latency at scale."
3. **Rationale**:
   - `tenant_id` is extracted from JWT-verified `VerifiedClaims` at the Oathkeeper
     boundary — the source is the verified identity, not caller-supplied data.
   - `bus.subscribe(&tenant_id)` scopes the channel to that tenant; no cross-tenant
     event delivery is possible by construction.
   - Keto governs visibility of *persistent entities* (event envelope objects).
     Agent events are ephemeral streaming frames, not Keto objects.
   - Subscribe-time Keto check (verify the tenant has the `agent_bus:stream` relation
     before opening the subscription) is added as the correct mitigation.
4. **Consequences**:
   - Subscription is cheaper (one Keto check at subscribe time, not per event).
   - Adds requirement for subscribe-time Keto check in `ws_agent_stream` and
     `agent_grpc_service.rs`.
   - Bidi streaming (future) will re-evaluate per-message Keto if message content
     contains object IDs subject to RLS.
5. **Alternatives considered**:
   - Per-event Keto check (rejected — unacceptable latency without an in-process
     cache; deferred to a future phase with a Redis-backed Keto cache).
   - Separate AuthZ middleware layer (deferred — requires middleware refactor).

## Files Affected

- `docs/decisions/adr-002-agent-bus-tenant-isolation.md` (NEW)

## Quality Gates

- [ ] ADR uses exact template from `adr-001-crdt-engine.md`
- [ ] Security deviation from CLAUDE.md is explicitly acknowledged and justified
- [ ] Subscribe-time Keto check is specified as the required mitigation
- [ ] `cargo check --workspace` continues to pass (no code changes)
