# p14-c005 — Add `ENABLE_FEDERATION_STAGE` gate to Phase 6 Layer 3 tests

> Phase: phase-14-stage10-dind-live-triage · Priority: MEDIUM

## Problem

Phase 6 Layer 3 tests require Matrix and ATProto bridges (`MATRIX_HOMESERVER_URL`, `ATPROTO_JETSTREAM_URL`) that are not in the Stage 10 compose stack. Without this gate, the tests run and time out waiting for events that never arrive on the WebSocket — consuming several minutes and producing misleading failure output.

## Solution

Add an `ENABLE_FEDERATION_STAGE` env var check to the Phase 6 Layer 3 `describe` block. When unset (or `"false"`), the block is skipped with an explanatory message.

```typescript
const ENABLE_FEDERATION = process.env["ENABLE_FEDERATION_STAGE"] === "true";

describe("Layer 3 — Federation bus (Matrix + ATProto)", () => {
  test.skip(!ENABLE_FEDERATION, "set ENABLE_FEDERATION_STAGE=true to run bridge tests");
  // ... existing tests ...
});
```

Stage 10 does NOT set `ENABLE_FEDERATION_STAGE` — the default is `false`, so federation tests are skipped. Future: when Tuwunel/Tranquil are in compose, set `ENABLE_FEDERATION_STAGE=true` in the Stage 10 container env.

## Files Changed

- `admin-ui/e2e/phase6-smoke.spec.ts` — add `ENABLE_FEDERATION_STAGE` guard to Layer 3 describe block

## Acceptance Criteria

- [ ] Layer 3 federation tests skip when `ENABLE_FEDERATION_STAGE` is unset
- [ ] Layer 3 tests run when `ENABLE_FEDERATION_STAGE=true`
- [ ] Stage 10 does not set `ENABLE_FEDERATION_STAGE` (tests skip by default)
- [ ] Skip message clearly states how to opt-in
- [ ] `pnpm typecheck` passes
