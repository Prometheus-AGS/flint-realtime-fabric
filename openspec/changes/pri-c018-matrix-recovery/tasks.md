# Tasks — pri-c018-matrix-recovery

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c007-broker-replay.
- [ ] 2. Complete the bounded change: Reuse actual Matrix sync/send code. Repair demonstrated direction/cursor/echo defects and stale stub logs; qualify the accepted Tuwunel/Matrix version.
- [ ] 3. Establish and record acceptance A: Inbound and outbound events reach intended rooms/channels after bridge restart with bounded deduplication and durable cursor continuity.
- [ ] 4. Establish and record acceptance B: Echo-loop suppression, cross-tenant denial, target outage and unconfigured direction errors are asserted against the actual selected local server version.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c018-matrix-recovery/`; run applicable local quality checks and `openspec validate pri-c018-matrix-recovery --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
