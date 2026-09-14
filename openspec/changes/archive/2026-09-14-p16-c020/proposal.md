# p16-c020 — Semantic config validation at boot

## Status: DONE

## Goal
G4.4 (#36) — phase-16-production-hardening

## Problem
Config validation checks only env-var presence, not semantic validity — e.g. the hosted SFU boots with empty LiveKit credentials.

## Solution
1. Validate semantic validity at startup
2. Fail fast with a clear message on invalid config (e.g. hosted SFU + empty LiveKit creds)

## Files Changed
- `crates/frf-gateway/src/config.rs`
- `crates/frf-gateway/src/main.rs`

## Risk
LOW.
