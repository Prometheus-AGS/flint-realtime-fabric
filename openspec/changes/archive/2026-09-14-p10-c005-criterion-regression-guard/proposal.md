# p10-c005 — Criterion Regression Guard Script

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Add `scripts/bench-regression-check.sh` that reads `target/criterion/` estimate
files, compares mean vs the committed baseline in `.criterion/`, and exits
non-zero if any benchmark regresses more than 10%. Wire the script into Dagger
Stage 9 after the bench run.

## Files to Create/Modify

- `scripts/bench-regression-check.sh` — jq-based regression threshold check
- `dagger/codegen.ts` — Stage 9: add the script execution after the bench run
- `scripts/` directory — create if not present (no other scripts currently)

## Design Notes

Criterion 0.5 stores estimates in
`target/criterion/<group>/<benchmark>/new/estimates.json`. The `mean.point_estimate`
field holds nanoseconds. The committed baseline is the same structure at
`.criterion/<group>/<benchmark>/main/estimates.json` (copied after
`--save-baseline main` in Phase 9).

Algorithm:
```bash
for baseline_file in .criterion/**/*/*/estimates.json; do
  bench_path="${baseline_file/.criterion\//target/criterion/}"
  bench_path="${bench_path/\/main\//\/new\/}"
  baseline_mean=$(jq '.mean.point_estimate' "$baseline_file")
  current_mean=$(jq '.mean.point_estimate' "$bench_path")
  pct=$(awk "BEGIN { printf \"%.1f\", (($current_mean - $baseline_mean) / $baseline_mean) * 100 }")
  if awk "BEGIN { exit ($pct <= 10.0) }"; then
    echo "REGRESSION: $bench_path regressed ${pct}% (baseline: ${baseline_mean}ns, current: ${current_mean}ns)"
    exit 1
  fi
done
echo "All benchmarks within 10% threshold"
```

Dagger Stage 9 addition — after bench run completes:
```typescript
.withExec(["bash", "scripts/bench-regression-check.sh"])
```

## Exit Criteria

- `scripts/bench-regression-check.sh` is executable and exits 0 when benches
  are within 10% of baseline
- Script exits 1 with a meaningful message when a benchmark exceeds 10%
  regression
- Dagger Stage 9 (`ENABLE_BENCH_STAGE=true`) runs the script after bench and
  fails the stage on regression
- `bash scripts/bench-regression-check.sh` passes locally (current benches
  match the committed baseline)
