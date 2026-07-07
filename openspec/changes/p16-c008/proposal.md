# p16-c008 — Wire admin-UI to gateway end to end

## Status: DONE

## Goal
G2.2 (C2/C3/H9/H10/H11) — phase-16-production-hardening

## Problem
The admin UI cannot reach the gateway: no Connect/gRPC-web endpoint for SpineService, no /ws/v1/signal route, gateway does not embed admin-ui, there is no login flow (empty JWT), and the default UI gateway URL/port mismatches the gateway bind.

## Solution
1. Serve Connect/gRPC-web for SpineService (tonic_web + accept_http1)
2. Add the real /ws/v1/signal route
3. Embed admin-ui via rust-embed/ServeDir
4. Add a login/auth flow so UI calls carry a real JWT
5. Reconcile the default gateway URL/port across UI and gateway

## Files Changed
- `crates/frf-gateway/src/lib.rs`
- `crates/frf-gateway/src/main.rs`
- `crates/frf-gateway/src/routes/`
- `admin-ui/src/infrastructure/`
- `admin-ui/src/features/auth/`

## Risk
MEDIUM — spans gateway transport + UI; verify with an E2E connection test.
