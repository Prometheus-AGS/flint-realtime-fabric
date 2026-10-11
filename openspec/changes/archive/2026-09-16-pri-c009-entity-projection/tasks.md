# Tasks — pri-c009-entity-projection

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c008-cdc-commit-mapping.
- [x] 2. Complete the bounded change: Compose a durable entity projection and population/checkpoint seam following one-port-per-adapter. Establish snapshot-to-WAL catch-up and readiness so existing GetEntity/WatchEntity reflect production ingestion.
- [x] 3. Establish and record acceptance A: A database commit reaches existing v1 GetEntity and WatchEntity through real adapters; snapshot overlap does not omit or duplicate state incorrectly.
- [x] 4. Establish and record acceptance B: Gateway/projector restart preserves state and cursor coherence; delete removes current state; v1 wire remains unchanged and required per-event object authorization is applied.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c009-entity-projection/`; run applicable local quality checks and `openspec validate pri-c009-entity-projection --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
