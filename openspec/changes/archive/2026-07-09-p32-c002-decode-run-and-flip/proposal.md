# p32-c002-decode-run-and-flip

## Why

Phase-32 G2: with the browser reaching the gateway over a genuine HTTPS secure context (c001 Caddy
sidecar), `getUserMedia` should succeed and the whole media path finally runs end-to-end. Observe
`framesDecoded > 0`. Flip `SFU_MODE=sovereign` only on a genuine pass; else re-affirm gated — and if
the blocker is again harness/environment, invoke the environment-pivot recommendation.

## What Changes

- Re-run `scripts/run-media-decode.sh` (HTTPS via caddy); record the real result in
  `docs/PHASE-32-DECODE-RESULT.md`, including `ice=`/`Connected`/`MediaData` + gateway str0m logs.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch + SECURITY §6
  (media → functional) + CHANGELOG + `docs/PHASE-32-SIGNOFF.md`. **Else:** re-affirm gated with the
  concrete detail + the environment-pivot recommendation if it's another harness/VM layer. G3 carried.

## Impact

- `docs/PHASE-32-DECODE-RESULT.md`, and (on a pass) `main.rs` + SECURITY §6 + CHANGELOG +
  PHASE-32-SIGNOFF. Gate flip conditional on a genuine decoded frame.
