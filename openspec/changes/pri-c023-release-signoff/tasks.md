# Tasks — pri-c023-release-signoff

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c001-build-policy, pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c006-watch-contract, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection, pri-c010-type-watch, pri-c011-forge-watch, pri-c012-sdk-pem-network, pri-c013-sync-persistence, pri-c014-aso-memory, pri-c015-aso-protected-proof, pri-c016-operations-recovery, pri-c017-hosted-media, pri-c018-matrix-recovery, pri-c019-atproto-recovery, pri-c020-sovereign-decode, pri-c021-admin-auth, pri-c022-platform-parity.
- [ ] 2. Complete the bounded change: Re-audit effective source/configuration, verify all selected profile receipts and publish a dated local GO/NO-GO matrix. Reconcile capability, rollout/rollback and support documentation; retain separate phase, certification and publication statuses.
- [ ] 3. Establish and record acceptance A: Every selected profile has exact repository/diff/lockfile/package/image/config/topology fingerprints, commands, timestamps, exits, scenario counts and artifact hashes; critical/high findings are resolved by an independent reviewer.
- [ ] 4. Establish and record acceptance B: A narrower profile may receive a scoped interim verdict while other capabilities remain blocked; c023 and the whole phase cannot close until all agreed phase goals are satisfied or the operator explicitly accepts scope reduction. No registry publication/deployment is inferred.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c023-release-signoff/`; run applicable local quality checks and `openspec validate pri-c023-release-signoff --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
