# Tasks — p6-c001: ADR-002 Agent Bus Tenant Isolation

- [ ] Read `docs/decisions/adr-001-crdt-engine.md` to capture exact template structure
- [ ] Write `docs/decisions/adr-002-agent-bus-tenant-isolation.md` with:
  - Status: Accepted
  - Context section: high-frequency ephemeral events, CLAUDE.md per-event RLS requirement
  - Decision: subscription-scoped `tenant_id` isolation + subscribe-time Keto check
  - Rationale: JWT-verified source, channel scoping by construction, Keto for persistent entities
  - Consequences: subscribe-time Keto check required in `ws_agent_stream` and `agent_grpc_service`
  - Alternatives: per-event Keto (rejected), middleware layer (deferred)
- [ ] Verify `cargo check --workspace` still passes (no code changes in this change)
