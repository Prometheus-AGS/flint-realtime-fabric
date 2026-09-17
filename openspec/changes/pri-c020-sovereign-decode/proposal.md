# Close sovereign media decode and protected fan-out

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C13. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Reproduce the zero-decoded-frame failure in a locally controlled Linux ICE/TURN topology. Trace room/track/keyframe routing, apply the smallest demonstrated correction and prove room-scoped protected egress.

## Ownership and Dependencies

Owner: Fabric str0m media. Depends on: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
Decision inputs before product implementation: Q1, Q2.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-media-str0m/src/`
- `crates/frf-gateway/src/`
- `scripts/`

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
