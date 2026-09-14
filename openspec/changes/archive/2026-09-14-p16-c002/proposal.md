# p16-c002 — App-layer tenant-equality assertion

## Status: DONE

## Goal
G1.2 (H1) — phase-16-production-hardening

## Problem
PublishUseCase and SubscribePipeline read the caller's tenant_id from the verified JWT but never compare it to the tenant_id on the target channel/envelope. A valid tenant-A caller can act on a tenant-B object if any grant exists.

## Solution
1. On publish: reject when claims.tenant_id != channel/envelope.tenant_id
2. On subscribe: reject when claims.tenant_id != channel.tenant_id
3. Return a typed Forbidden error; add tests for match + mismatch

## Files Changed
- `crates/frf-app/src/publish.rs`
- `crates/frf-app/src/subscribe.rs`
- `crates/frf-app/src/error.rs`

## Risk
LOW — additive guard beneath Keto; covered by unit tests.
