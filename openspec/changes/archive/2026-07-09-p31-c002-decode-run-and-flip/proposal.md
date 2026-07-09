# p31-c002-decode-run-and-flip

## Why

Phase-31 G2: with the runner no longer building the gateway image in-run (c001), run the in-network
decode against a pre-built image on a restarted daemon. This is the first LIVE verification of the
phase-30 topology fix (`ice=connected`). Observe `framesDecoded > 0`. Flip `SFU_MODE=sovereign` only
on a genuine pass; else re-affirm gated.

## What Changes

- Re-run `scripts/run-media-decode.sh` (pre-built image, restarted VM); record the real result in
  `docs/PHASE-31-DECODE-RESULT.md`, including whether `ice=connected` / `state=Connected` /
  `MediaData` (the phase-30 UNVERIFIED G1 exit) + gateway logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-31-SIGNOFF.md`. **Else:** re-affirm gated with the
  concrete detail. G3 carried.

## Impact

- `docs/PHASE-31-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-31-SIGNOFF. Gate flip conditional on a genuine decoded frame.
