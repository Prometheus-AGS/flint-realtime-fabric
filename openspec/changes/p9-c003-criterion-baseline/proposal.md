# p9-c003 — Criterion Baseline Commit + CI Bench Stage

## Summary

Run `cargo bench -p frf-crdt -- --save-baseline main`; commit `.criterion/` baseline JSON;
add optional Dagger Stage 9 (`bench`) gated on `ENABLE_BENCH_STAGE=true`.

## Files to Create/Modify

- `.criterion/crdt_merge/baseline.json` — committed Criterion baseline (generated)
- `.gitignore` — allow `.criterion/`, ignore `target/criterion/`
- `dagger/codegen.ts` — add Stage 9 bench step

## Exit Criteria

- `.criterion/crdt_merge/baseline.json` exists and is committed
- `dagger/codegen.ts` typechecks
- `cargo bench -p frf-crdt` runs without error
