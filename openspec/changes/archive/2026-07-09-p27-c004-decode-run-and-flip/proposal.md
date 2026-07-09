# p27-c004-decode-run-and-flip

## Why

Phase-27 G3: with the three diagnosed defects fixed — bidirectional trickle ICE (c002) + RoomJoin
fan-out (c003), confirmed by instrumentation (c001) — re-run the authenticated decode proof and
observe `framesDecoded > 0`. Flip `SFU_MODE=sovereign` **only** on a genuine pass; else re-affirm
gated with the fresh (instrumented) detail.

## What Changes

- Execute `scripts/run-media-decode.sh` (rebuilds the gateway image to pick up the c001/c002 Rust
  changes); record the real outcome in `docs/PHASE-27-DECODE-RESULT.md` — including the instrumented
  ICE state / candidate counts / str0m lifecycle.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + `docs/SECURITY.md` §6 (media → functional) + CHANGELOG +
  `docs/PHASE-27-SIGNOFF.md`. **Else:** re-affirm gated with the concrete detail. + G4 carried.

## Impact

- `docs/PHASE-27-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-27-SIGNOFF. Gate flip is conditional on a genuine decoded frame.
