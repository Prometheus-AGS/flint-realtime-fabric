# Tasks — pri-c021-admin-auth

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
- [ ] 2. Complete the bounded change: Implement only the admin profile accepted in c003: either a deliberately restricted operator tool or the separately accepted interactive login design. Reuse the actual Gate/IdP surface without assuming an unrelated identity provider.
- [ ] 3. Establish and record acceptance A: Selected admin login/token entry, expiry, logout and unauthorized access behavior work against real local identity services.
- [ ] 4. Establish and record acceptance B: Interactive-login claims require the actual redirect/session flow; a documented operator-only profile is never reported as delivered interactive login.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c021-admin-auth/`; run applicable local quality checks and `openspec validate pri-c021-admin-auth --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
