# p14-c006 — Commit `.wasm-size-baseline` after DinD Stage 6 run

> Phase: phase-14-stage10-dind-live-triage · Priority: MANUAL

## Problem

The `.wasm-size-baseline` file does not exist. Without it, the wasm size gate in Stage 6 is disabled (the check skips gracefully). The baseline must be seeded from an actual DinD build of the WASM target.

## Solution

This is a **manual operator step** — it cannot be performed without a DinD environment.

### Procedure

```bash
# 1. Ensure compose stack is running and Stage 6 can build
make compose-up

# 2. Run Stage 6 to produce the WASM binary
ENABLE_INTEGRATION_STAGE=true dagger call stage6 \
  --src . \
  --output ./target/wasm-build

# 3. Measure and commit the baseline
make baseline-wasm
# This runs: wasm-opt --print-size <path> | tee .wasm-size-baseline
# and commits the file

# 4. Verify commit
git log --oneline -1
```

The `make baseline-wasm` target was created in Phase 13 (`Makefile`).

## Files Changed

- `.wasm-size-baseline` — NEW FILE (created by `make baseline-wasm` after DinD run)

## Acceptance Criteria

- [ ] `.wasm-size-baseline` exists in the repo root
- [ ] File contains a numeric byte count
- [ ] File is committed to git
- [ ] Stage 6 wasm size gate checks against the baseline on subsequent runs

## Note

This change CANNOT be applied in a session without Docker / DinD. The operator must run `make baseline-wasm` manually after a successful Stage 6 DinD build, then commit the result.
