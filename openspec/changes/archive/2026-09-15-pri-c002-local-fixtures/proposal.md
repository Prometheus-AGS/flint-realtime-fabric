# Provide an isolated local integration runner

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C09. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Create a namespaced composed fixture and mandatory-prerequisite runner with disposable credentials, deadlines, receipts and ownership-safe cleanup. Replace the mux outline with real publish/receive; reserve CDC semantics for c008.

## Ownership and Dependencies

Owner: Fabric integration. Depends on: pri-c001-build-policy.
Decision inputs before product implementation: none beyond dependencies.
Status: implementation and local verification complete; archive pending.

## Impact

Affected modules and repository-relative paths:
- `compose.integration.yml`
- `Makefile`
- `README.md`
- `docs/DEVELOPMENT.md`
- `scripts/`
- `crates/frf-gateway/tests/subscribe_mux.rs`
- `crates/frf-postgres-cdc/tests/cdc_integration.rs`

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
