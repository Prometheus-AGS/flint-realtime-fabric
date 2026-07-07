# p16-c018 — /metrics Prometheus endpoint

## Status: DONE

## Goal
G4.2 (#34) — phase-16-production-hardening

## Problem
The gateway emits tracing spans but exposes no /metrics endpoint, so it is unobservable in production.

## Solution
1. Add a Prometheus /metrics endpoint
2. Export core request + delivery metrics

## Files Changed
- `crates/frf-gateway/src/lib.rs`
- `crates/frf-gateway/src/routes/`
- `crates/frf-gateway/Cargo.toml`

## Risk
LOW — additive endpoint.
