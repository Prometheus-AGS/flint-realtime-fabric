# p16-c011 — Create frf-sdk-rust hand-written client

## Status: DONE

## Goal
G3.1 (C5) — phase-16-production-hardening

## Problem
The canonical hand-written Rust SDK (the single home for connection lifecycle and CRDT merge) does not exist. No other SDK has a reference to inherit.

## Solution
1. Create crates/frf-sdk-rust with connect/publish/subscribe
2. Wrap the gateway transport (Connect/gRPC/WS)
3. Expose a clean typed client API

## Files Changed
- `crates/frf-sdk-rust/`
- `Cargo.toml`

## Risk
MEDIUM — new crate; foundational for c012/c014.
