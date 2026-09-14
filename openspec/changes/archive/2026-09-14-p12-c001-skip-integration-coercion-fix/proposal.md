# p12-c001 — Fix SKIP_INTEGRATION Coercion Bug

## Phase
phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Summary

Fix the boolean coercion bug in the `SKIP_INTEGRATION` guard across all three
Layer 3 Playwright smoke specs. `!!process.env["SKIP_INTEGRATION"]` treats the
string `"false"` as truthy (any non-empty string is truthy in JS), causing all
integration tests to skip even when Stage 10 explicitly sets
`SKIP_INTEGRATION=false`.

## Files to Modify

- `admin-ui/e2e/phase4-smoke.spec.ts` line 19
- `admin-ui/e2e/phase5-smoke.spec.ts` line 20
- `admin-ui/e2e/phase6-smoke.spec.ts` line 21

In each file, change:
```typescript
// WRONG — truthy string coercion
const SKIP_INTEGRATION = !!process.env["SKIP_INTEGRATION"] || !process.env["GATEWAY_URL"];
```
to:
```typescript
// CORRECT — explicit string comparison
const SKIP_INTEGRATION = process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
```

## Design Notes

The pattern `!!process.env["X"]` is correct when `X` is expected to be either
unset (undefined → false) or set to any value (truthy). It is WRONG when `X`
is set to the string `"false"` as an explicit opt-out signal — because `"false"`
is a truthy string.

The canonical pattern in this codebase for an opt-in integration flag is:
`process.env["X"] === "true"` — only the exact string "true" enables it, and
"false" or unset both disable it.

## Exit Criteria

- `grep -r "SKIP_INTEGRATION" admin-ui/e2e/` shows `=== "true"` pattern in phase4/5/6 specs
- TypeScript typecheck passes after edit
- Layer 1 tests in all three specs still pass (SKIP_INTEGRATION unset = tests skip as expected)
- With `GATEWAY_URL=x SKIP_INTEGRATION=false`, integration test blocks are entered (skip message no longer shown)
