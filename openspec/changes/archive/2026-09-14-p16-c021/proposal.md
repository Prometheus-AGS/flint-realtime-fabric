# p16-c021 — Keto schema-migration step

## Status: DONE

## Goal
G4.5 (#37) — phase-16-production-hardening

## Problem
There is no Keto schema-migration step in compose; persistent-DSN deployments will not start without a manual `keto migrate`.

## Solution
1. Add a keto migrate init step to compose
2. Document the migration in the runbook

## Files Changed
- `compose.yml`
- `compose.ci.yml`
- `docs/`

## Risk
LOW.
