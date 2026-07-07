# p16-c017 — Real /readyz readiness probe

## Status: DONE

## Goal
G4.1 (H8) — phase-16-production-hardening

## Problem
/healthz is a static stub that never probes dependencies; there is no /readyz. A fully-broken gateway reports healthy, so K8s/LB keeps a dead pod in rotation.

## Solution
1. Add /readyz that checks Iggy/Keto/JWKS liveness
2. Keep /healthz as liveness
3. Gate compose/K8s depends_on on readiness

## Files Changed
- `crates/frf-gateway/src/routes/health.rs`
- `compose.yml`
- `compose.ci.yml`

## Risk
LOW — additive probe.
