# Qualify the protected shape-to-SQL-to-PEM path

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C08. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Prove current-source protected replication using the existing shape feed and materializer, independently of generic entity watching. Cover every selected browser/native topology and approved schema/egress boundary.

## Ownership and Dependencies

Owner: ASO + Gate/Fabric + PEM. Depends on: pri-c005-authority-lifetime, pri-c014-aso-memory.
Decision inputs before product implementation: Q2, Q7.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/`
- `crates/frf-app/src/shape/`
- `crates/frf-gateway/src/routes/shape.rs`
- `../flint-gate/`

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
