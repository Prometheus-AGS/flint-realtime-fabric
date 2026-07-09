# p33-c002-decode-run-and-flip

## Why

Phase-33 G3: with both peers on the VM host network (c001), the gateway's advertised host candidate
and the browser's are on one routable stack — a pair should form. Run the host-net decode. Observe
`framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a genuine pass; else re-affirm gated and, if
host candidates still don't pair, recommend the TURN fallback (Target C).

## What Changes

- Re-run `HOST_NET=1 scripts/run-media-decode.sh`; record the real result in
  `docs/PHASE-33-DECODE-RESULT.md`, including `ice=`/`Connected`/`MediaData` + gateway str0m logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-33-SIGNOFF.md`. **Else:** re-affirm gated + TURN
  fallback recommendation. G4 carried.

## Impact

- `docs/PHASE-33-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-33-SIGNOFF. Gate flip conditional on a genuine decoded frame.
