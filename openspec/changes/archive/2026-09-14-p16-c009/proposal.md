# p16-c009 — Mark Matrix/ATProto federation unsupported for v1

## Status: DONE

## Goal
G2.5 (#10/#27) — phase-16-production-hardening

## Problem
Matrix inbound is hardcoded stream::empty() and ATProto outbound send() is a hard Err, yet both are wired as bidirectional bridges — advertising capability that does not exist. Per scope decision, federation protocol work is DEFERRED for v1.

## Solution
1. Stop wiring the broken directions as bidirectional
2. Return a clear 'unsupported in v1' rather than silent empty/Err
3. Where cheap, apply configured tenant/channel IDs (G2.4) instead of random per-boot values
4. Document federation as deferred

## Files Changed
- `crates/frf-gateway/src/main.rs`
- `crates/frf-bridge-matrix/src/`
- `crates/frf-bridge-atproto/src/`
- `docs/`

## Risk
LOW — honesty + de-wiring; no new protocol code.
