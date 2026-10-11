# Tasks — pri-c003-scope-research-contract

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: none.
- [x] 2. Complete the bounded change: Resolve Q1–Q7 with provenance and research the named identity/policy, persistence/deployment, media/federation and platform backlog in bounded passes. Inventory source/package/image revisions and runtime profiles. Record Q8 separately; KBD migration is not part of this change. Research and decision facilitation may start immediately; unresolved Q1–Q7 block closure and dependent implementation, not the start of this change.
- [x] 3. Establish and record acceptance A: An accepted release matrix names required profiles, clients, source schemas/keys/tenants, historical/delete semantics, scale/retention/RTO/RPO, ASO budgets and profile-specific authorization lifetime endpoints.
- [x] 4. Establish and record acceptance B: Pinned API, compatibility and maintenance/security evidence exists for each retained dependency; unsupported choices trigger a revised proposal, not invented feasibility. Reconcile Assess provenance finding through focused independent review.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c003-scope-research-contract/`; run applicable local quality checks and `openspec validate pri-c003-scope-research-contract --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
