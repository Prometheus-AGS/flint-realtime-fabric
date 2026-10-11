# Tasks — pri-c008-cdc-commit-mapping

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c006-watch-contract, pri-c007-broker-replay.
- [x] 2. Complete the bounded change: Reuse the pinned WAL parser; implement validated catalog/replica-identity mapping, schema-qualified routing and supported typed values. Couple acknowledged WAL progress to durable committed publication, including decode/transaction failures.
- [x] 3. Establish and record acceptance A: Real INSERT/UPDATE/DELETE and rollback/multi-row transactions exercise enrolled non-first/composite/non-UUID keys and accepted tenant modes; unsupported mappings fail enrollment, not silent delivery.
- [x] 4. Establish and record acceptance B: Crash between publication and LSN advancement permits defined deduplication without loss; poison rows, schema change, unchanged TOAST and missing old-key data cannot be skipped beneath an acknowledged checkpoint.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c008-cdc-commit-mapping/`; run applicable local quality checks and `openspec validate pri-c008-cdc-commit-mapping --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched hand-authored files remain
at most 500 lines. Machine-generated dependency lockfiles retain their required
native package-manager format and are source-bound separately.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
