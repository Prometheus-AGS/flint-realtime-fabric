# Tasks — pri-c020-sovereign-decode

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
- [ ] 2. Complete the bounded change: Reproduce the zero-decoded-frame failure in a locally controlled Linux ICE/TURN topology. Trace room/track/keyframe routing, apply the smallest demonstrated correction and prove room-scoped protected egress.
- [ ] 3. Establish and record acceptance A: Repeated two-peer campaigns show increasing receiver framesDecoded plus audio progress, including late join/reconnect and multiple rooms; ICE/RTP alone is insufficient.
- [ ] 4. Establish and record acceptance B: Revocation/expiry removes protected fan-out within c003's accepted bound; wrong-room/tenant traffic is denied and task/socket counts return to baseline. Keep production mode gated until proof.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c020-sovereign-decode/`; run applicable local quality checks and `openspec validate pri-c020-sovereign-decode --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
