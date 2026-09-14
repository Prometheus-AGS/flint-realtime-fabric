# p16-c012 — Reconnection/backoff/replay in SDK

## Status: DONE

## Goal
G3.3 (H6) — phase-16-production-hardening

## Problem
No SDK implements reconnection; a transient blip or gateway restart ends the subscribe stream permanently. Resumable-offset primitives exist in proto but are unused.

## Solution
1. Implement reconnection with backoff in frf-sdk-rust
2. Replay-from-offset using resumable-offset proto primitives
3. Surface the behavior through TS/Go/C#

## Files Changed
- `crates/frf-sdk-rust/`
- `sdks/ts/`
- `sdks/go/`
- `sdks/csharp/`

## Risk
MEDIUM — depends-on c011.
