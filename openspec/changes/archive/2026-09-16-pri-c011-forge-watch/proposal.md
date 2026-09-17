# Replace Forge's Fabric stub with safe GraphQL delivery

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C06. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Use the tonic Rust client generated/exported and compile-checked in c010 and canonical payload mapping. Deliver the authorized RLS re-query projection; implement the accepted safe deletion/invalidation contract and truncation reconstruction. Retain LISTEN default until parity passes.

## Ownership and Dependencies

Owner: Forge realtime/app/gateway. Depends on: pri-c005-authority-lifetime, pri-c010-type-watch.
Decision inputs before product implementation: none beyond dependencies.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `../flint-forge/crates/fdb-realtime/src/`
- `../flint-forge/crates/fdb-app/src/lib.rs`
- `../flint-forge/crates/fdb-gateway/src/realtime_source.rs`

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
