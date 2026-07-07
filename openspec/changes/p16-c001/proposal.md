# p16-c001 — Remove auth bypass from production compose

## Status: DONE

## Goal
G1.1 (C1) — phase-16-production-hardening

## Problem
compose.yml is production-shaped (real Keto, flint-gate JWKS, JWT_AUDIENCE, live CDC) yet builds the gateway with `CARGO_FEATURES: dev-endpoints` and sets `DEV_NO_AUTH: "true"`, compiling in and activating a total JWT/Cedar auth bypass. Anyone lifting compose.yml as a prod template ships a gateway that accepts empty tokens on /v1/publish and /ws/v1/subscribe.

## Solution
1. Remove `CARGO_FEATURES: dev-endpoints` and `DEV_NO_AUTH` from compose.yml
2. Keep the bypass ONLY in compose.ci.yml and compose.override.yml (dev/CI paths)
3. Make `dev_no_auth()` and the bypass branches strictly `#[cfg(feature="dev-endpoints")]` so a default release build (no features) cannot reach them
4. Confirm the default Dockerfile build produces a secure image

## Files Changed
- `compose.yml`
- `crates/frf-gateway/src/config.rs`
- `crates/frf-gateway/src/routes/publish.rs`
- `crates/frf-gateway/src/routes/subscribe.rs`

## Risk
LOW — config + compile-gating; verifiable by building the default image and asserting publish/subscribe require a token.
