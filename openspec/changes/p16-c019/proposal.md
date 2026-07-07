# p16-c019 — Graceful shutdown on SIGTERM

## Status: DONE

## Goal
G4.3 (#35) — phase-16-production-hardening

## Problem
The axum server has no graceful shutdown; in-flight requests and WS streams are dropped on SIGTERM.

## Solution
1. Install a shutdown signal handler
2. Drain in-flight requests and WS streams before exit

## Files Changed
- `crates/frf-gateway/src/main.rs`

## Risk
LOW.
