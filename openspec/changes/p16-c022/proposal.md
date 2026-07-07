# p16-c022 — Env-var reference + .env.example

## Status: DONE

## Goal
G5.1 (H14) — phase-16-production-hardening

## Problem
~30 gateway env vars (including secrets and the DEV_NO_AUTH bypass) are entirely undocumented; there is no .env.example.

## Solution
1. Document every gateway env var: purpose, required?, secret?, dev-only?
2. Provide a .env.example

## Files Changed
- `docs/`
- `.env.example`

## Risk
LOW.
