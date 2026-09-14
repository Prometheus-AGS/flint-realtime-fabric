# p13-c005 — Commit `.wasm-size-baseline` (manual step)

## Summary

This is a **manual step** that cannot be automated without a DinD-capable
environment. After c001–c004 are merged and a successful Stage 6 run
produces `sdks/ts/frf-wasm/frf_wasm_bg.wasm`, the operator measures the
binary size and commits it as `.wasm-size-baseline` to arm the regression
guard introduced in Phase 12 (c005).

## Prerequisite

- Changes c001–c004 must be merged and a full Dagger pipeline run must
  succeed through Stage 6.

## Operator procedure

```bash
# After a successful Stage 6 run (wasm-pack build completed):
SIZE=$(wc -c < sdks/ts/frf-wasm/frf_wasm_bg.wasm | tr -d ' ')
echo "$SIZE" > .wasm-size-baseline
git add .wasm-size-baseline
git commit -m "chore: establish WASM binary size baseline (${SIZE} bytes)"
git push origin main
```

Alternatively, use the `make baseline-wasm` target added in c006.

## Files

- `.wasm-size-baseline` — new file, single integer (byte count of `frf_wasm_bg.wasm`)

## Acceptance criteria

1. `.wasm-size-baseline` contains a single positive integer.
2. Stage 6 WASM size gate emits `OK: WASM size <N> within 150% of baseline`
   on the subsequent run.
3. A future commit that increases the WASM binary size by >50% causes
   Stage 6 to fail with `FAIL: WASM size > 150% of baseline`.
