# Deliver the runnable PGlite–Forge–Electric prototype loop

## Why

Prototype applications need a complete local-first data loop before broad SDK
packaging work. Fabric and Forge foundations are useful only when a user can
mutate local PGlite state, persist through Forge, receive the canonical database
row through Electric/Fabric and see PEM update without manual refresh.

## What Changes

Build a checked-in runnable browser/Node prototype. PGlite stores optimistic
local state, Forge's authenticated mutation API writes PostgreSQL, Electric
streams committed database rows, Fabric's authorized shape facade protects that
stream, PGlite electric sync reconciles the local table, and PEM observes the
resulting local-table change. Broad native/generated SDK packaging follows only
after this path works.

## Ownership and Dependencies

Owner: Forge + Fabric shape facade + PGlite/PEM adapter. Depends on: pri-c004-deployment-profiles, pri-c005-authority-lifetime and implemented c011 tasks 2–3. The deferred c011 combined-campaign receipt does not block this implementation.
Decision inputs before product implementation: none beyond dependencies.
Status: implementation active; baseline task complete and the online loop is next.

## Impact

Affected modules and repository-relative paths:
- `crates/frf-shape-electric/`
- `crates/frf-gateway/src/routes/shape.rs`
- `sdks/ts/`
- `../prometheus-entity-sync/packages/entity-sync-pglite/`
- `../flint-forge/crates/fdb-gateway/`
- runnable local compose/example assets in the owning repositories

Dependency-rule impact: keep domain/application imports inward; adapter behavior
is accessed through ports, with one port per adapter and gateway composition.
Changes to public domain/port types require a documented semver decision.

## Foundation Compatibility

Preserve the completed Phase 0 foundation: workspace build and immutable v1
contract. This is a readiness follow-up; it does not recreate or claim Phase 0.

## Non-goals

Native SDK parity, offline retry durability, unrelated CRDT persistence,
automatic production deployment/publication and KBD runtime migration are
outside this slice. c013 adds offline durability without replacing this online
prototype path.
