# p16-c004 — Verify JWT issuer (iss)

## Status: DONE

## Goal
G1.4 (#25) — phase-16-production-hardening

## Problem
The identity verifier validates the JWT signature/audience but does not verify the issuer (iss), accepting tokens from any issuer whose key resolves.

## Solution
1. Add expected-issuer config (env)
2. Validate iss against the expected value during verify()
3. Reject tokens with missing/mismatched iss; add tests

## Files Changed
- `crates/frf-identity-ory/src/verifier.rs`
- `crates/frf-gateway/src/config.rs`

## Risk
LOW — additive validation with tests.
