# Tasks — pri-c005-authority-lifetime

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles.
- [x] 2. Complete the bounded change: Verify real Gate credentials and current object authorization on generic streams, and preserve Gate/ASO fresh authority on restricted shape streams. Check expiry, rotation, revoked grants, cancellation and bypass paths.
- [x] 3. Establish and record acceptance A: Real Gate-issued tokens allow only intended issuer/audience/tenant/subject and object; same-tenant unauthorized subjects, cross-tenant access, authority loss, key rotation and stale refill races are exercised locally.
- [x] 4. Establish and record acceptance B: ASO authoritative commit/expiry to last server-produced protected frame or cancellation preventing the next frame is at most 5000 ms, followed by new-request denial; direct Kratos revocation uses the documented observation/ASO-denial start. All other lanes meet the lifetime contract accepted in c003.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c005-authority-lifetime/`; run applicable local quality checks and `openspec validate pri-c005-authority-lifetime --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
