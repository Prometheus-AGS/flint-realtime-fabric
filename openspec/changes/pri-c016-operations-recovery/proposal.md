# Qualify capacity, recovery and rotation per profile

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C10. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Execute load and failure campaigns at accepted profile targets; cover broker/WAL/checkpoint lag, backup/restore, rolling replacement, secrets/certificate rotation, drains and no-progress alerts. Final qualification cannot close before every selected profile implementation and client surface is complete; early diagnostic campaigns are non-final.

## Ownership and Dependencies

Owner: Deployment + Fabric operations. Depends on: pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection, pri-c010-type-watch, pri-c011-forge-watch, pri-c012-sdk-pem-network, pri-c013-sync-persistence, pri-c014-aso-memory, pri-c015-aso-protected-proof, pri-c017-hosted-media, pri-c018-matrix-recovery, pri-c019-atproto-recovery, pri-c020-sovereign-decode, pri-c021-admin-auth, pri-c022-platform-parity.
Decision inputs before product implementation: Q2, Q6.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `compose.yml`
- `k8s/`
- `scripts/`
- `docs/`

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
