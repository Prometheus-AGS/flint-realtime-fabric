# Tasks — pri-c010-type-watch

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c005-authority-lifetime, pri-c006-watch-contract, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection.
- [x] 2. Complete the bounded change: Implement the frozen type-watch use case and transport using durable broker delivery and projection semantics. Enforce subscribe-time/per-event authority, bounded buffers, explicit resync and teardown. Generate and export the frozen-version tonic Rust client from frf-proto, and compile the Forge-facing client surface here before c011; c012 owns separate non-Rust package generation.
- [x] 3. Establish and record acceptance A: Two authorized subscribers receive committed changes of the enrolled type; other schemas/types/tenants and unauthorized same-tenant subjects receive no protected payload.
- [x] 4. Establish and record acceptance B: Snapshot/subscribe races, reconnect, expired cursor, slow-client lag, cancellation during awaited authorization and revocation meet the contract; resources return to baseline.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c010-type-watch/`; run applicable local quality checks and `openspec validate pri-c010-type-watch --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
