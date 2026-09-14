# p16-c010 — Defer str0m; make LiveKit the v1 media path

## Status: DONE

## Goal
G2.1/G2.3 (C4/H4) — phase-16-production-hardening

## Problem
str0m 'sovereign SFU' has no real WebRTC (str0m crate never imported). Per scope decision, sovereign mode is DEFERRED; LiveKit-hosted is the only supported v1 media path. LiveKit currently never listens to the server, so cross-node signals are lost.

## Solution
1. Gate SFU_MODE=sovereign off by default; document as unimplemented/future
2. Fix LiveKit to listen to the server so cross-node inbound signals arrive (H4)
3. Document LiveKit-hosted as the supported v1 path

## Files Changed
- `crates/frf-gateway/src/main.rs`
- `crates/frf-media-livekit/src/adapter.rs`
- `docs/`

## Risk
MEDIUM — LiveKit listen path is real work; str0m is a gating/doc change.
