# Populate durable v1 entity reads and watches

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C05 / F01. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Compose a durable entity projection and population/checkpoint seam following one-port-per-adapter. Establish snapshot-to-WAL catch-up and readiness so existing GetEntity/WatchEntity reflect production ingestion.

## Ownership and Dependencies

Owner: Fabric entity projection. Depends on: pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c008-cdc-commit-mapping.
Decision inputs before product implementation: none beyond dependencies.
Status: implemented and locally verified; all implementation tasks are complete.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-gateway/src/main.rs`
- `crates/frf-app/src/entity.rs`
- `crates/frf-ports/src/`
- `crates/frf-projection-surreal/src/`

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
