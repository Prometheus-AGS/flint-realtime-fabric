# Tasks — pri-c007-broker-replay

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c002-local-fixtures, pri-c006-watch-contract.
- [x] 2. Complete the bounded change: Replace producer-local replay counters with the accepted broker position mapping. Configure explicit commit policy instead of polling-time defaults and prove exact pinned-fork seek/replay/ack semantics, stable producer identity and cancellation.
- [x] 3. Establish and record acceptance A: Crash after poll but before durable consumer checkpoint replays the event; restart after acknowledgement respects defined cursor inclusivity and duplicate rules.
- [x] 4. Establish and record acceptance B: Producer restart does not reuse a replay position; independent consumers, partition boundaries, retention expiry and cancellation behave as the frozen contract requires. Record actual server durability/flush configuration.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c007-broker-replay/`; run applicable local quality checks and `openspec validate pri-c007-broker-replay --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
