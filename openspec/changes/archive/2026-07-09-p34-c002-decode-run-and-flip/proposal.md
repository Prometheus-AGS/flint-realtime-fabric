# p34-c002-decode-run-and-flip

## Why

Phase-34 G2: with a TURN relay on the bridge (c001), both browsers gather a `typ relay` candidate
(a routable IP str0m accepts) — the routable pair phase-32 couldn't form. Run the decode. Observe
`framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a genuine pass; else re-affirm gated + CI pivot.

## What Changes

- Re-run `scripts/run-media-decode.sh` (bridge + TURN); record the real result in
  `docs/PHASE-34-DECODE-RESULT.md` (`ice=`/`Connected`/`MediaData`/relay-pairing + gateway str0m logs).
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-34-SIGNOFF.md`. **Else:** re-affirm gated + CI-pivot
  (Target B) recommendation. G3 carried.

## Impact

- `docs/PHASE-34-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-34-SIGNOFF. Gate flip conditional on a genuine decoded frame.
