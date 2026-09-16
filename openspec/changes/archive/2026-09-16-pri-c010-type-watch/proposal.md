# Serve authorized resumable WatchEntityType streams

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C05. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Implement the frozen type-watch use case and transport using durable broker delivery and projection semantics. Enforce subscribe-time/per-event authority, bounded buffers, explicit resync and teardown. Generate and export the frozen-version tonic Rust client from frf-proto, and compile the Forge-facing client surface here before c011; c012 owns separate non-Rust package generation.

## Ownership and Dependencies

Owner: Fabric application + gateway. Depends on: pri-c005-authority-lifetime, pri-c006-watch-contract, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection.
Decision inputs before product implementation: none beyond dependencies.
Status: implemented and locally verified; all implementation tasks and review
findings are resolved in the source-bound c010 evidence.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-app/src/`
- `crates/frf-gateway/src/`
- `crates/frf-proto/`

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
