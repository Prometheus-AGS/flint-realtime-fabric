# p30-c002-decode-run-and-flip

## Why

Phase-30 G2: with the browser now inside the Docker network (c001), the browser and SFU share one
address space (the Colima VM bridge), so a candidate pair should complete. Re-run the decode proof.
Observe `framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a genuine pass; else re-affirm gated.

## What Changes

- Re-run `scripts/run-media-decode.sh` (in-network browser + shared socket + coturn); record the real
  result in `docs/PHASE-30-DECODE-RESULT.md`, including diagnostics + gateway logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-30-SIGNOFF.md`. **Else:** re-affirm gated with the
  concrete detail. G3 carried.

## Impact

- `docs/PHASE-30-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-30-SIGNOFF. Gate flip conditional on a genuine decoded frame.
