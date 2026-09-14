# p16-c023 — Deployment/operations runbook

## Status: DONE

## Goal
G5.2 (H15) — phase-16-production-hardening

## Problem
Only local-compose docs exist. There is no production runbook: topology, secrets, CDC-slot lifecycle/recovery, scaling, upgrade/rollback. A stalled slot pins WAL and fills Postgres disk with no guidance.

## Solution
1. Write a prod deployment + operations runbook
2. Cover secret provisioning, CDC-slot lifecycle/recovery, scaling, upgrade/rollback

## Files Changed
- `docs/`

## Risk
LOW.
