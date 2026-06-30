# Execution — Phase 6: Federation + Matrix/ATProto Bridge

> Started: 2026-06-20 · Backend: openspec · Tool: kbd-execute

## Backend Selection

**openspec** — `openspec/` directory present; `project.json` confirms
`"change_backend": "openspec"`. All 7 changes tracked in
`openspec/changes/p6-c00{1-7}-*/`.

## Dispatch Contract

- Changes execute in dependency order (see plan.md dependency graph)
- p6-c001 + p6-c002 run first (no mutual deps)
- p6-c003 + p6-c004 + p6-c005 run in parallel after p6-c002 completes
- p6-c006 is the sync point (all three prior must complete)
- p6-c007 is the final gate

## Quality Gate Protocol

After each change: `cargo check --workspace` → `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` → change-specific tests.

After p6-c006: additionally `cd admin-ui && pnpm typecheck && pnpm build`.
After p6-c007: additionally `SKIP_INTEGRATION=true pnpm exec playwright test e2e/phase6-smoke.spec.ts`.

## Change Status

| Change | Status | Notes |
|---|---|---|
| p6-c001-adr-agent-bus-tenant | PENDING | — |
| p6-c002-librefang-tenant-registry | PENDING | — |
| p6-c003-frf-bridge-matrix | PENDING | awaits p6-c002 |
| p6-c004-frf-bridge-atproto | PENDING | awaits p6-c002 |
| p6-c005-agent-grpc-service | PENDING | awaits p6-c002 |
| p6-c006-gateway-federation-wiring | PENDING | awaits c003+c004+c005 |
| p6-c007-e2e-federation-smoke | PENDING | awaits c006 |
