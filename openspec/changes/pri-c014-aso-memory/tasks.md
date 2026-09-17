# Tasks — pri-c014-aso-memory

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract.
- [ ] 2. Complete the bounded change: Reconcile current source with mounted receipts and profile the existing 16203-row workload. Apply measured, bounded memory corrections while preserving SQL/checkpoint atomicity, owner fencing and the experimental gate.
- [ ] 3. Establish and record acceptance A: Same-fixture browser campaign measures incremental RSS at or below 536870912 bytes and heap at or below 268435456 bytes, unless c003 records an explicit operator-approved replacement contract.
- [ ] 4. Establish and record acceptance B: Worker/DB/graph-copy and cold-fold allocations explain the result; passing behavior survives optimization. If one bounded session cannot close the gate, record findings and split remaining implementation before continuing; do not close this change on diagnosis alone. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/`; run applicable local quality checks and `openspec validate pri-c014-aso-memory --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
