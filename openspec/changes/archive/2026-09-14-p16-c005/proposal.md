# p16-c005 — Wire rate limiting, body limit, CORS

## Status: DONE

## Goal
G1.5 (H2) — phase-16-production-hardening

## Problem
The gateway router has zero tower-http layers: no rate limiting, no request body-size limit, and no CORS policy on a browser-facing service. tower-http is declared but unused.

## Solution
1. Add a rate-limit layer
2. Add a RequestBodyLimit layer
3. Add an explicit CorsLayer policy (allowed origins/methods/headers)

## Files Changed
- `crates/frf-gateway/src/lib.rs`
- `crates/frf-gateway/Cargo.toml`

## Risk
LOW — middleware wiring; validate with an integration test.
