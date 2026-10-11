# Tasks — pri-c002-local-fixtures

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c001-build-policy.
- [x] 2. Complete the bounded change: Create a namespaced composed fixture and mandatory-prerequisite runner with disposable credentials, deadlines, receipts and ownership-safe cleanup. Replace the mux outline with real publish/receive; reserve CDC semantics for c008.
- [x] 3. Establish and record acceptance A: A real authenticated publish reaches a subscribed gateway client; disabling delivery makes that assertion fail before restoration passes.
- [x] 4. Establish and record acceptance B: Missing prerequisites, zero required scenarios and skipped required tests exit nonzero; cleanup touches only the run-owned containers/volumes/slots.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c002-local-fixtures/`; run applicable local quality checks and `openspec validate pri-c002-local-fixtures --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
