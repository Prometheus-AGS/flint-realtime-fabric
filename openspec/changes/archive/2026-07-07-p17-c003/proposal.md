# p17-c003 — Author constraints.md + wire QA gate

## Status: PROPOSED

## Why

Phase-16 never ran the artifact-refiner QA gate (no .refiner/, no constraints.md). Author .kbd-orchestrator/constraints.md and wire /refine-validate per-change into the execute loop so every phase-17 change is validated before archive.

Phase: phase-17-plane-completion-and-release-audit · Goal: G2
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
