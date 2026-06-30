# Plan — Phase 5: Agent Protocols + BossFang (LibreFang fork)

> Created: 2026-06-20 · Change backend: OpenSpec

---

## Change Backend

OpenSpec detected (`openspec/` directory present, `project.json` confirms `"change_backend": "openspec"`).
All 6 changes are tracked in `openspec/changes/p5-c00{1-6}-*/`.

---

## Ordering Rationale

The dependency chain is strictly sequential — no parallelism is possible:

```
p5-c001  AgentEventBus port trait        (frf-ports)
   ↓ required by
p5-c002  frf-agentproto crate            (depends on trait + domain types)
   ↓ required by
p5-c003  frf-librefang crate             (implements AgentEventBus with ractor)
   ↓ required by
p5-c004  Gateway AgentService + wiring   (composes frf-librefang into AppState)
   ↓ required by
p5-c005  Admin UI agents feature         (consumes /ws/v1/agents WebSocket)
   ↓ tests all of the above
p5-c006  E2E agent smoke tests           (Phase 5 exit criterion)
```

---

## Change List

| # | ID | Title | Crate/Layer | Est. Complexity |
|---|---|---|---|---|
| 1 | `p5-c001-agent-event-bus-port` | AgentEventBus port trait | `frf-ports` | Low |
| 2 | `p5-c002-frf-agentproto` | frf-agentproto crate (ContentBlock) | `frf-agentproto` (new) | Medium |
| 3 | `p5-c003-frf-librefang` | BossFang actor bus adapter | `frf-librefang` (new) | High |
| 4 | `p5-c004-gateway-agent-service` | Gateway AgentService + AppState B generic | `frf-gateway` | High |
| 5 | `p5-c005-admin-ui-agents` | Admin UI agent activity panel | `admin-ui` | Medium |
| 6 | `p5-c006-e2e-agent-smoke` | Playwright E2E exit criterion tests | `admin-ui/e2e` | Low |

---

## Open Decisions (must resolve at p5-c003 kickoff)

1. **BossFang fork source** — git dep or crates.io. If crates.io, confirm package name.
   This is the first task inside p5-c003 before any code is written.
2. **Actor tree granularity** — single supervisor vs per-tenant. Recommendation: start
   with a single tree; per-tenant sharding is a future optimization if load requires it.
3. **AG-UI spec version** — ContentBlock schema in p5-c002 should be validated against
   the upstream AG-UI spec. Check `https://ag-ui.com` or the repo for current event types.

---

## Security Constraints (from CLAUDE.md — binding on all 6 changes)

- Tenant isolation enforced at Keto layer — NOT in actor bus or agent service code.
- JWT verification via Oathkeeper at gateway boundary; `tenant_id` extracted from
  `VerifiedClaims`, never from user-supplied query params or body.
- Cedar governs action policy (mutating ops); Keto governs visibility. Agent event
  subscriptions are same-tenant visibility — document in ADR if departing from Keto.
- Never log JWT payloads, tenant identifiers, or relation tuples in debug output.

---

## Quality Gates (each change must pass before the next begins)

- `cargo check --workspace` clean
- `cargo clippy --workspace -- -D warnings -W clippy::pedantic` clean
- No `unwrap()` in library crates (`frf-ports`, `frf-agentproto`, `frf-librefang`)
- `pnpm typecheck` clean (TypeScript changes)
- Tests pass for each new crate

---

## First Change

**Start with**: `/kbd-execute phase-5-agent-protocols-bossfang` → apply `p5-c001-agent-event-bus-port`
