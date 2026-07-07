# p16-c007 — Make Cedar functional or explicit no-op

## Status: DONE

## Goal
G1.7 (H3) — phase-16-production-hardening

## Problem
Cedar runs with Entities::empty() + a blanket permit-all default policy, so it is structurally incapable of real ABAC: silent allow-all by default, or silent deny-all with any real attribute policy (no entities supplied).

## Solution
1. Supply real Entities to the authorizer, OR
2. Gate Cedar off explicitly with a clear 'policy engine disabled' log
3. No silent allow-all / deny-all path

## Files Changed
- `crates/frf-policy-cedar/src/lib.rs`
- `policy.cedar`

## Risk
MEDIUM — either real ABAC wiring or an explicit disabled mode; must be honest.
