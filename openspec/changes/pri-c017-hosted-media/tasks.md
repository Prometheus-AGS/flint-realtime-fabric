# Tasks — pri-c017-hosted-media

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
- [ ] 2. Complete the bounded change: Reuse researched LiveKit integration; prove real two-participant decoded audio/video and grant lifetime on the selected server version. Qualify relay only if advertised.
- [ ] 3. Establish and record acceptance A: Two clients show progressing decoded audio/video across late join and reconnect; wrong room/tenant/subject is denied and revoked participants stop under the accepted lifetime contract.
- [ ] 4. Establish and record acceptance B: Any advertised cross-node relay passes a two-gateway campaign; absent configurations fail visibly. Local protocol fixtures alone cannot certify an untested production service version.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c017-hosted-media/`; run applicable local quality checks and `openspec validate pri-c017-hosted-media --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
