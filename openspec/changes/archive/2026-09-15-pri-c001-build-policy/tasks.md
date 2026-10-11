# Tasks — pri-c001-build-policy

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: none.
- [x] 2. Complete the bounded change: Fix the demonstrated missing SignalEnvelope subject and reconcile MSRV. Remove CI-reachable test execution while preserving local entry points; correct stale Dart-generator guidance to ADR-003.
- [x] 3. Establish and record acceptance A: Default, dev-endpoints and shape-facade checks plus applicable Clippy, format and SDK typecheck succeed locally against recorded source hashes.
- [x] 4. Establish and record acceptance B: Inventory every workflow/Dagger call path: CI performs build/lint/typecheck/format/package only; each removed runtime gate has a documented local invocation and is exercised by its owning later change.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c001-build-policy/`; run applicable local quality checks and `openspec validate pri-c001-build-policy --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
