# Freeze the versioned watch and recovery contract

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C04. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Specify a new versioned WatchEntityType contract, canonical typed payload/key mapping and compatibility with immutable v1. Separate source LSN/epoch, stable event identity, partition offset and client checkpoint; specify snapshot barrier, ordering, duplicates, retention, lag, cancellation and authorized deletes.

## Ownership and Dependencies

Owner: Fabric contract + Forge/PEM consumers. Depends on: pri-c003-scope-research-contract.
Decision inputs before product implementation: Q4, Q5, Q6.
Status: implementation, deterministic local verification, required review
finding resolution and independent resolution review complete; archive pending.

## Impact

Affected modules and repository-relative paths:
- `proto/flint/`
- `crates/frf-domain/src/`
- `crates/frf-ports/src/`
- `docs/decisions/`

Dependency-rule impact: keep domain/application imports inward; adapter behavior
is accessed through ports, with one port per adapter and gateway composition.
Changes to public domain/port types require a documented semver decision.

## Foundation Compatibility

Preserve the completed Phase 0 foundation: workspace build and immutable v1
contract. This is a readiness follow-up; it does not recreate or claim Phase 0.

## Non-goals

Unrelated refactoring, automatic production deployment/publication, and KBD
runtime migration are outside this proposal. Full readiness requires the phase's
other accepted gates; this slice cannot certify them.
