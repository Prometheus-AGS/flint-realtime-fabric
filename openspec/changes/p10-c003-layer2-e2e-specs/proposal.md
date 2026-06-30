# p10-c003 — Admin UI Layer 2 E2E Spec Files

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Add three new Layer 2 Playwright spec files covering publish form-fill,
WebSocket subscribe stream, and agent session lifecycle. All Layer 2 tests
gate on `SKIP_INTEGRATION !== 'true'` and require gateway at `GATEWAY_URL`.

## Files to Create/Modify

- `admin-ui/e2e/layer2-publish.spec.ts` — form-fill publish POST, expects
  success/auth response logged in UI
- `admin-ui/e2e/layer2-subscribe.spec.ts` — entity stream WS connect;
  connected badge turns green in `EntityGraph`
- `admin-ui/e2e/layer2-agent.spec.ts` — `AgentActivityPanel` WS session
  start → events stream → stop lifecycle

## Design Notes

Follow the 3-layer convention established in `p7-smoke.spec.ts`:

```typescript
const skipIntegration = process.env.SKIP_INTEGRATION === 'true';

test.describe('Layer 2: Publish (requires gateway)', () => {
  test.skip(skipIntegration, 'Set SKIP_INTEGRATION=false to run Layer 2 tests');
  // ...
});
```

Gateway URL from `process.env.GATEWAY_URL ?? 'http://localhost:8080'`.

**layer2-publish.spec.ts** targets `/#demo/signaling` → fills in the
publish form fields, clicks Submit, asserts the UI shows a success/error
state (not a blank response).

**layer2-subscribe.spec.ts** targets `/` (EntitiesPage → EntityGraph) →
waits for the `data-testid="connection-badge"` to have class `connected`
within 10s.

**layer2-agent.spec.ts** targets `/#agents` → clicks Start Session →
waits for at least one event row to appear in `AgentActivityPanel` →
clicks Stop Session → asserts session ended state.

## Exit Criteria

- `pnpm --filter @prometheusags/frf-admin-ui exec playwright test --list`
  lists all 3 new spec files
- `SKIP_INTEGRATION=true pnpm e2e` exits 0 (Layer 2 tests skipped)
- With gateway running: `SKIP_INTEGRATION=false pnpm e2e` Layer 2 tests pass
- No TypeScript errors in the 3 new spec files
