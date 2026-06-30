# p11-c004 — CRDT Layer 2 Test

## Phase
phase-11-layer3-e2e-wasm-opt-cdc

## Summary

Add `admin-ui/e2e/layer2-crdt.spec.ts` with Layer 1 (UI shape) and Layer 2
(WASM round-trip) tests for the `crdt_apply_delta` function exported by
`frf-wasm`. The Layer 2 test executes `crdt_apply_delta` via Playwright's
`page.evaluate()` against the already-loaded frf-wasm module in the browser.

## Files to Create/Modify

- `admin-ui/e2e/layer2-crdt.spec.ts` (NEW)

## Design Notes

`crdt_apply_delta(snapshot: Uint8Array, delta: Uint8Array): Uint8Array` is
exported by the built `frf_wasm.js`. The function returns the original snapshot
when either input is empty (no-op path), making it testable without a live
gateway.

Layer 1 test strategy (no gateway required):
- Navigate to `/#demo/signaling`
- Verify the CRDT Apply Delta button is visible
- Verify the button is disabled when no delta input is provided

Layer 2 test strategy (gated on `SKIP_INTEGRATION=false && WASM_AVAILABLE=1`):
- Use `page.evaluate()` to call `window.crdt_apply_delta` if injected, or
  import `frf_wasm.js` dynamically via `page.addScriptTag`
- Call `crdt_apply_delta(new Uint8Array([]), new Uint8Array([]))` — empty
  inputs return empty snapshot (no-op), result should be `Uint8Array` length 0
- Call `crdt_apply_delta(new Uint8Array([1,2,3]), new Uint8Array([]))` — empty
  delta returns snapshot unchanged, result length should be 3

Gating:
```typescript
const skipCrdt = process.env["SKIP_INTEGRATION"] === "true"
  || !process.env["WASM_AVAILABLE"];
```

## Exit Criteria

- `layer2-crdt.spec.ts` registered in `pnpm exec playwright test --list`
- Layer 1 tests pass with `SKIP_INTEGRATION=true pnpm e2e`
- Layer 2 tests are properly skipped when `WASM_AVAILABLE` is unset
- TypeScript typecheck passes after the new file is added
