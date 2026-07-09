# p35-c002-decode-run-and-flip

## Why

Phase-35 G3: run the decode on GitHub Actions `ubuntu-latest` (c001's `decode-proof` job) where host
networking is native and the six local candidate-address confusions don't arise. Observe
`framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a genuine pass; else re-affirm gated.

## What Changes

- Trigger the `decode-proof` workflow (`workflow_dispatch`); record the real result (job conclusion +
  the browser assertion + gateway str0m logs from the artifacts) in `docs/PHASE-35-DECODE-RESULT.md`.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-35-SIGNOFF.md`. **Else:** re-affirm gated with the CI
  diagnostics. G4 carried.

## Impact

- `docs/PHASE-35-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-35-SIGNOFF. Gate flip conditional on a genuine decoded frame.
