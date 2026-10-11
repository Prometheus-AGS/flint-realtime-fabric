# Restore the declared build matrix and local-only testing policy

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C01. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Fix the demonstrated missing SignalEnvelope subject and reconcile MSRV. Remove CI-reachable test execution while preserving local entry points; correct stale Dart-generator guidance to ADR-003.

## Ownership and Dependencies

Owner: Fabric. Depends on: none.
Decision inputs before product implementation: none beyond dependencies.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `Cargo.toml`
- `.github/workflows/ci.yml`
- `dagger/`
- `crates/frf-gateway/src/routes/dev.rs`
- `openspec/config.yaml`

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
