# Tasks — pri-c006-watch-contract

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c003-scope-research-contract.
- [x] 2. Complete the bounded change: Specify a new versioned WatchEntityType contract, canonical typed payload/key mapping and compatibility with immutable v1. Separate source LSN/epoch, stable event identity, partition offset and client checkpoint; specify snapshot barrier, ordering, duplicates, retention, lag, cancellation and authorized deletes.
- [x] 3. Establish and record acceptance A: Contract scenarios cover accepted schema/key/tenant shapes, transaction commit, snapshot-to-live races, restart/replay, old cursors, unauthorized history and deletion/projection; consumer acceptance is recorded before codegen.
- [x] 4. Establish and record acceptance B: Frozen proto/flint/v1 files remain byte-identical; new-version generation and old/new compatibility validation succeed. Semver impact on frf-domain/frf-ports and one-port adapter boundaries are documented.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c006-watch-contract/`; run applicable local quality checks and `openspec validate pri-c006-watch-contract --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
