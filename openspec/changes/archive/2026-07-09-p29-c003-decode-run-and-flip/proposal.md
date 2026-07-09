# p29-c003-decode-run-and-flip

## Why

Phase-29 G3: with B2 (shared demuxing socket, c001) and B1 (STUN srflx + `.local` skip, c002) both
cleared, re-run the decode proof. Observe `framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a
genuine pass; else re-affirm gated with the fresh diagnostics.

## What Changes

- Re-run `scripts/run-media-decode.sh` (boots coturn + the shared-socket gateway); record the real
  result in `docs/PHASE-29-DECODE-RESULT.md`, including diagnostics + gateway logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-29-SIGNOFF.md`. **Else:** re-affirm gated with the
  concrete detail. G4 carried.

## Impact

- `docs/PHASE-29-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-29-SIGNOFF. Gate flip conditional on a genuine decoded frame.
