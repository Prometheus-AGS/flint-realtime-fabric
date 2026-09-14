# p5-c006 — E2E agent smoke tests (Phase 5 exit criterion)

## Summary

Playwright E2E tests covering the Phase 5 exit criterion: an AG-UI agent event
flows through BossFang → browser WebSocket consumer and appears in the admin UI.

## Test Layers (same 3-layer pattern as Phase 4)

### Layer 1 — UI (no gateway required, CI-safe)

- AgentActivityPanel renders in empty state
- Panel shows "No agent events" placeholder
- Navigation to agents tab works
- WS disconnect does not crash the panel

### Layer 2 — WS layer (gated on `GATEWAY_URL`)

- WS upgrade to `/ws/v1/agents` returns 101
- Tenant extracted from JWT claim header (mock claim header in test)

### Layer 3 — End-to-end (gated on `GATEWAY_URL`)

- POST a synthetic `AgentRunRequest` via gRPC reflection OR REST shim
- Wait for `data-testid="agent-event-row"` to appear in the panel
- Assert `kind` badge text equals `run_start`
- Assert `run_end` event appears within 5s

## Files Changed

- `admin-ui/e2e/phase5-smoke.spec.ts` — NEW (3-layer pattern, same skip logic as phase4)

## Acceptance Criteria

- [ ] All UI-layer tests pass without gateway (`pnpm test:e2e`)
- [ ] WS-layer and E2E tests are skipped unless `GATEWAY_URL` is set
- [ ] No flaky timeout assertions — use `expect(locator).toBeVisible()` waits
- [ ] Tests use `data-testid` selectors, not CSS class or text
