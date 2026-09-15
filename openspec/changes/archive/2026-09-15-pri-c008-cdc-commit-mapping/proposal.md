# Deliver committed typed CDC changes without checkpoint gaps

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C05. This change supplies one bounded
acceptance slice and its source-bound local qualification.

## What Changes

Reuse the pinned WAL parser; implement validated catalog/replica-identity mapping, schema-qualified routing and supported typed values. Couple acknowledged WAL progress to durable committed publication, including decode/transaction failures.

## Ownership and Dependencies

Owner: Fabric CDC. Depends on: pri-c002-local-fixtures, pri-c006-watch-contract, pri-c007-broker-replay.
Decision inputs before product implementation: none beyond dependencies.
Status: implemented; both acceptance outcomes pass against the bound local
PostgreSQL 17 and Iggy fixture, and all independent findings are resolved.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-postgres-cdc/src/consumer.rs`
- `crates/frf-postgres-cdc/src/decode.rs`
- `crates/frf-postgres-cdc/src/catalog.rs`
- `crates/frf-postgres-cdc/src/canonical.rs`
- `crates/frf-gateway/tests/cdc_integration.rs`
- `crates/frf-gateway/src/config/`
- `compose.cdc-integration.yml`
- `scripts/run-cdc-integration.sh`

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
