# Qualify required native and generated SDK transports

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C14. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Generate using pinned or explicitly superseded ADR-003 tooling. Address Dart async transport only through a demonstrated compatible approach; qualify every required install/runtime/device surface.

## Ownership and Dependencies

Owner: Fabric SDK/FFI + consumer owners. Depends on: pri-c003-scope-research-contract, pri-c006-watch-contract, pri-c012-sdk-pem-network, pri-c013-sync-persistence.
Decision inputs before product implementation: Q3.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-ffi/`
- `sdks/go/`
- `sdks/csharp/`
- `sdks/swift/`
- `sdks/kotlin/`
- `sdks/dart/`

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
