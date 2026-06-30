# Execution — Phase 5: Agent Protocols + BossFang

> Created: 2026-06-20 · Backend: openspec

## Backend

`openspec` — changes tracked in `openspec/changes/p5-c00{1-6}-*/`

## Dispatch Contract

Changes execute in strict dependency order. Each change must pass
`cargo check --workspace` (Rust) or `pnpm typecheck` (TS) before the next begins.

## Change Order

1. `p5-c001-agent-event-bus-port` — AgentEventBus trait in frf-ports
2. `p5-c002-frf-agentproto` — ContentBlock crate
3. `p5-c003-frf-librefang` — BossFang actor bus adapter (requires BossFang URL decision)
4. `p5-c004-gateway-agent-service` — AppState<B> + AgentServiceImpl
5. `p5-c005-admin-ui-agents` — Admin UI agent activity panel
6. `p5-c006-e2e-agent-smoke` — Playwright E2E exit criterion

## Status

Active change: p5-c001-agent-event-bus-port
