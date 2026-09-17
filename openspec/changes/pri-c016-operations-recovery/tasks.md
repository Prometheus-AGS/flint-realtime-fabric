# Tasks — pri-c016-operations-recovery

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection, pri-c010-type-watch, pri-c011-forge-watch, pri-c012-sdk-pem-network, pri-c013-sync-persistence, pri-c014-aso-memory, pri-c015-aso-protected-proof, pri-c017-hosted-media, pri-c018-matrix-recovery, pri-c019-atproto-recovery, pri-c020-sovereign-decode, pri-c021-admin-auth, pri-c022-platform-parity.
- [ ] 2. Complete the bounded change: Execute load and failure campaigns at accepted profile targets; cover broker/WAL/checkpoint lag, backup/restore, rolling replacement, secrets/certificate rotation, drains and no-progress alerts. Final qualification cannot close before every selected profile implementation and client surface is complete; early diagnostic campaigns are non-final.
- [ ] 3. Establish and record acceptance A: Local campaigns meet accepted subscription/change-rate/latency/retention/RTO/RPO targets with explicit resource allocation, no required skips and recorded recovery positions.
- [ ] 4. Establish and record acceptance B: Restoration and credential/certificate rotation preserve required authorization and durable state; bounded backpressure/cancellation and alerts prevent false-ready/no-progress operation. Repeat affected receipts after consumer or profile changes.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c016-operations-recovery/`; run applicable local quality checks and `openspec validate pri-c016-operations-recovery --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
