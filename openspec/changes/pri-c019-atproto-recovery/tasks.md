# Tasks — pri-c019-atproto-recovery

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c007-broker-replay.
- [ ] 2. Complete the bounded change: Reuse existing firehose and configured outbound implementations. Prove stable cursor/channel identity, authorization and retry behavior against selected Tranquil/PDS versions.
- [ ] 3. Establish and record acceptance A: Supported directions resume after disconnect/process restart without missed acknowledged events or unbounded duplication.
- [ ] 4. Establish and record acceptance B: Outbound retries, self-echo, tenant/channel routing and unavailable/unconfigured targets have explicit assertions; faithful protocol mocks are labeled and cannot substitute for target-version qualification.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c019-atproto-recovery/`; run applicable local quality checks and `openspec validate pri-c019-atproto-recovery --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
