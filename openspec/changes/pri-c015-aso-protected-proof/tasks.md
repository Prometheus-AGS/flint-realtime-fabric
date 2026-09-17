# Tasks — pri-c015-aso-protected-proof

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c005-authority-lifetime, pri-c014-aso-memory.
- [ ] 2. Complete the bounded change: Prove current-source protected replication using the existing shape feed and materializer, independently of generic entity watching. Cover every selected browser/native topology and approved schema/egress boundary.
- [ ] 3. Establish and record acceptance A: Crash before SQL checkpoint rolls back rows/checkpoints; crash after SQL commit before PEM publication recovers coherently only after current authorization. Logout/practice switch fences stale owners and clears the old graph.
- [ ] 4. Establish and record acceptance B: Interrupted shape bodies, expiry/refetch, revocation timing, direct-backend isolation and forbidden/local-only field egress pass for each selected topology; no competing generic-watch writer handles the same clinical rows. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c015-aso-protected-proof/`; run applicable local quality checks and `openspec validate pri-c015-aso-protected-proof --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
