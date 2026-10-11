# Make PGlite outbound sync durable and restart-safe

## Why

The online c012 loop must remain usable through loss of connectivity and client
restart. Prototype applications cannot treat a locally accepted PGlite mutation
as durable if it disappears or duplicates its database effect during retry.

## What Changes

Add a durable PGlite outbox, stable mutation identity, retry/replay and canonical
reconciliation around the c012 loop. Retire an outbox item only after canonical
database state returns through Electric/Fabric.

## Ownership and Dependencies

Owner: PGlite/PEM sync transport + Forge mutation API. Depends on functional completion of c012 tasks 2–3; c012's deferred combined-campaign receipt does not block implementation.
Decision inputs before product implementation: none beyond dependencies.
Status: source implementation complete; combined runtime campaign and evidence pending.

## Impact

Affected modules and repository-relative paths:
- `../prometheus-entity-sync/packages/entity-sync-pglite/`
- `../prometheus-entity-sync/packages/entity-sync-core/`
- `../flint-forge/crates/fdb-gateway/`
- `../flint-forge/crates/fdb-app/`
- runnable local compose/example assets in the owning repositories

Dependency-rule impact: keep domain/application imports inward; adapter behavior
is accessed through ports, with one port per adapter and gateway composition.
Changes to public domain/port types require a documented semver decision.

## Foundation Compatibility

Preserve the completed Phase 0 foundation: workspace build and immutable v1
contract. This is a readiness follow-up; it does not recreate or claim Phase 0.

## Non-goals

Fabric CRDT snapshot/op-store qualification, broad SDK parity, unrelated
refactoring, automatic production deployment/publication and KBD runtime
migration are outside this proposal. CRDT persistence is explicitly deferred;
this change does not claim it.
