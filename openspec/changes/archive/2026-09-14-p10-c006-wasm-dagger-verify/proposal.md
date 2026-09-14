# p10-c006 — WASM Dagger Stage 6 Output Verification

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Verify Dagger Stage 6 (`ENABLE_WASM_STAGE=true`) produces `frf_wasm.js` and
`frf_wasm_bg.wasm` at the expected output path; add an output existence check
to the Dagger stage so it fails explicitly when the WASM build is incomplete.

## Files to Create/Modify

- `dagger/codegen.ts` — Stage 6 wasm-build: add post-build verification step
  that checks `sdks/ts/frf-wasm/frf_wasm.js` and `frf_wasm_bg.wasm` exist

## Design Notes

The current Stage 6 runs:
```typescript
.withExec(["wasm-pack", "build", "--target", "web",
    "--out-dir", "../../sdks/ts/frf-wasm",
    "--out-name", "frf_wasm",
    "--release",
])
```

Add a verification step immediately after:
```typescript
.withExec(["sh", "-c",
    "test -f sdks/ts/frf-wasm/frf_wasm.js && " +
    "test -f sdks/ts/frf-wasm/frf_wasm_bg.wasm && " +
    "echo 'WASM output verified' || " +
    "{ echo 'WASM build output missing'; exit 1; }"
])
```

This catches silent `wasm-pack` failures where the process exits 0 but
produces no output (can happen with stale cargo caches or missing
wasm32-unknown-unknown target).

Also add a check that the generated `package.json` has `"name": "frf-wasm"`:
```typescript
.withExec(["sh", "-c",
    "jq -e '.name == \"frf-wasm\"' sdks/ts/frf-wasm/package.json || " +
    "{ echo 'frf-wasm package.json name mismatch'; exit 1; }"
])
```

Depends on p10-c001 (which runs `wasm-pack` locally first to confirm output
shape before wiring the Dagger verification).

## Exit Criteria

- `ENABLE_WASM_STAGE=true` Dagger Stage 6 fails explicitly when WASM outputs
  are missing
- Stage 6 passes when `frf_wasm.js`, `frf_wasm_bg.wasm`, and a correctly
  named `package.json` are present
- `dagger/codegen.ts` passes TypeScript typecheck after the edit
