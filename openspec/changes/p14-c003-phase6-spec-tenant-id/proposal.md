# p14-c003 — Add `tenant_id` to Phase 6 spec POST bodies

> Phase: phase-14-stage10-dind-live-triage · Priority: CRITICAL

## Problem

The Rust handler `InjectFederationEventRequest` requires `tenant_id: String` (a UUID). The Phase 6 smoke spec posts to `/dev/inject-federation-event` without `tenant_id`, causing serde to return a 422 Unprocessable Entity error before any test assertion runs.

## Solution

Add `tenant_id: "00000000-0000-0000-0000-000000000002"` to all POST bodies in `admin-ui/e2e/phase6-smoke.spec.ts` that target `/dev/inject-federation-event`.

The dev UUID `00000000-0000-0000-0000-000000000002` is consistent with the nil-UUID convention used elsewhere in Phase 6 Layer 1 tests.

Lines to update in `phase6-smoke.spec.ts`:

- Lines 149–156 (Matrix inject block)
- Lines 171–178 (ATProto inject block)

### Before

```typescript
data: {
    protocol: "matrix",
    source: "!smoke-room:matrix.org",
    content: { type: "text_delta", delta: `matrix smoke ${runId}` },
},
```

### After

```typescript
data: {
    tenant_id: "00000000-0000-0000-0000-000000000002",
    protocol: "matrix",
    source: "!smoke-room:matrix.org",
    content: { type: "text_delta", delta: `matrix smoke ${runId}` },
},
```

Same fix for the ATProto block.

## Files Changed

- `admin-ui/e2e/phase6-smoke.spec.ts` — add `tenant_id` to Matrix and ATProto inject POST bodies

## Acceptance Criteria

- [ ] Both inject POSTs include `tenant_id: "00000000-0000-0000-0000-000000000002"`
- [ ] TypeScript check passes: `pnpm typecheck`
- [ ] No other POST bodies are modified by this change
