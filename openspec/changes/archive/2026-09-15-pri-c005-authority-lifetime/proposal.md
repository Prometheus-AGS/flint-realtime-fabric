# Enforce identity and authorization on selected exposed lanes

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C03. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Verify real Gate credentials and current object authorization on generic streams, and preserve Gate/ASO fresh authority on restricted shape streams. Check expiry, rotation, revoked grants, cancellation and bypass paths.

## Ownership and Dependencies

Owner: Gate + Fabric; Forge/ASO boundary owners. Depends on: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles.
Decision inputs before product implementation: Q2, Q5, Q7.
Status: implementation, local verification and required review resolution are
complete; archive pending.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-gateway/src/`
- `crates/frf-app/src/`
- `crates/frf-authz-keto/src/`
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
