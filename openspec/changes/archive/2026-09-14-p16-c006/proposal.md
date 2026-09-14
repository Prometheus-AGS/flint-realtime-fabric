# p16-c006 — Externalize flint-gate signing secret

## Status: DONE

## Goal
G1.6 (#48) — phase-16-production-hardening

## Problem
The flint-gate JWT signing secret is committed in compose.yml. Secrets must not live in the repo.

## Solution
1. Move the secret to env / secret manager
2. Reference it via env in compose
3. Document rotation

## Files Changed
- `compose.yml`
- `compose.ci.yml`
- `docs/`

## Risk
LOW — config change; rotate the exposed value.
