# p25-c002-live-decode-run

## Why

Phase-25 G2 — the proof. c001 built the authenticated runner (RS256 mint + JWKS + clean sovereign
stack + seed). This change **executes it** against the live gateway and records the actual
`framesDecoded` outcome — the input that decides the flip (c003).

## What Changes

- Execute `scripts/run-media-decode.sh` (authenticated path) and record the real result in
  `docs/PHASE-25-DECODE-RESULT.md`: `framesDecoded > 0` observed → c003 flips; else the concrete
  blocker, diagnosed as ICE/DTLS/RTP (**media path** — the real unknown) vs. harness/infra. No
  fabricated pass.

## Impact

- New: `docs/PHASE-25-DECODE-RESULT.md`. No production code; no gate flip. The recorded outcome is
  c003's input.
