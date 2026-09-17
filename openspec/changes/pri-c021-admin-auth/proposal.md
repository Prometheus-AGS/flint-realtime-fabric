# Deliver the accepted admin authentication profile

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C14. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Implement only the admin profile accepted in c003: either a deliberately restricted operator tool or the separately accepted interactive login design. Reuse the actual Gate/IdP surface without assuming an unrelated identity provider.

## Ownership and Dependencies

Owner: Fabric admin + Gate. Depends on: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
Decision inputs before product implementation: Q1, Q2.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `admin-ui/`
- `crates/frf-gateway/src/`
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
