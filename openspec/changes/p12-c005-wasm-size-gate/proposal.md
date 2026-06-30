# p12-c005 — WASM Binary Size Gate

## Phase
phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Summary

Add a WASM binary size measurement and gate to Dagger Stage 6. After the
`wasm-pack build` step, measure `frf_wasm_bg.wasm` size and:
1. Emit the size in bytes to stdout (always)
2. Compare against a committed baseline in `.wasm-size-baseline` (if it exists)
3. Fail if size exceeds 150% of baseline (regression guard)
4. Commit the baseline file on first successful run

## Files to Create/Modify

- `dagger/codegen.ts` — Stage 6: add size gate step after the verification steps
- `.wasm-size-baseline` (NEW) — committed after first run, contains the measured byte count

### codegen.ts change

After the existing `jq -e '.name == "frf-wasm"'` verification step, add:

```typescript
// Measure WASM binary size; compare against committed baseline if it exists.
.withExec(["sh", "-c",
    "SIZE=$(wc -c < /workspace/sdks/ts/frf-wasm/frf_wasm_bg.wasm) && " +
    "echo \"WASM binary size: ${SIZE} bytes\" && " +
    "if [ -f /workspace/.wasm-size-baseline ]; then " +
    "  BASELINE=$(cat /workspace/.wasm-size-baseline); " +
    "  LIMIT=$((BASELINE * 3 / 2)); " +
    "  if [ \"$SIZE\" -gt \"$LIMIT\" ]; then " +
    "    echo \"FAIL: WASM size ${SIZE} > 150% of baseline ${BASELINE} (limit: ${LIMIT})\"; " +
    "    exit 1; " +
    "  else " +
    "    echo \"OK: WASM size ${SIZE} within 150% of baseline ${BASELINE}\"; " +
    "  fi; " +
    "else " +
    "  echo \"No baseline found — run: echo \\$SIZE > .wasm-size-baseline and commit\"; " +
    "fi"
])
```

### .wasm-size-baseline

Initial content: TBD (populated from first successful Stage 6 run). File is
committed to the repo root so CI always has a baseline to compare against.

The value is a raw byte count (integer, no units). Example:
```
524288
```

## Design Notes

`wc -c` counts bytes (not characters) and works on binary files. The 150%
threshold allows for legitimate growth from new CRDT features; it is NOT a
file size budget (see performance.md for that). The intent is to catch
accidental regressions like including debug symbols or forgetting to strip.

The baseline is updated by running `wasm-pack build` locally, noting the size,
and committing `.wasm-size-baseline` with the new value. A PR that bumps the
baseline should document why (e.g., "added Loro text CRDT document model:
+42KB").

## Exit Criteria

- Stage 6 emits `WASM binary size: N bytes` to stdout
- Stage 6 exits 0 when `.wasm-size-baseline` is absent (no baseline = first run)
- Stage 6 exits 0 when WASM size ≤ 150% of baseline
- Stage 6 exits 1 when WASM size > 150% of baseline with clear error message
- `.wasm-size-baseline` committed to repo root after first run
