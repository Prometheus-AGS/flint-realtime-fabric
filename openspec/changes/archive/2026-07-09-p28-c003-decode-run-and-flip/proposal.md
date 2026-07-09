# p28-c003-decode-run-and-flip

## Why

Phase-28 G3: with the ICE candidate-IP bug fixed (p28-c002 — advertise a valid `127.0.0.1` instead
of `0.0.0.0`), re-run the decode proof. Observe `framesDecoded > 0`. Flip `SFU_MODE=sovereign` only
on a genuine pass; else re-affirm gated with the fresh (now-visible) detail.

## What Changes

- Re-run `scripts/run-media-decode.sh` (rebuilds the gateway for the str0m change); record the real
  result in `docs/PHASE-28-DECODE-RESULT.md`, including the surfaced diagnostics + gateway logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + `docs/SECURITY.md` §6 (media → functional) + CHANGELOG +
  `docs/PHASE-28-SIGNOFF.md`. **Else:** re-affirm gated with the concrete detail. + G4 carried.

## Impact

- `docs/PHASE-28-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-28-SIGNOFF. Gate flip conditional on a genuine decoded frame.
