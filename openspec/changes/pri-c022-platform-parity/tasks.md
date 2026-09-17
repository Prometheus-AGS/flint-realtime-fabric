# Tasks — pri-c022-platform-parity

All items are planned. Do not check them off from a compile-only or historical receipt.

- [ ] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c003-scope-research-contract, pri-c006-watch-contract, pri-c012-sdk-pem-network, pri-c013-sync-persistence.
- [ ] 2. Complete the bounded change: Generate using pinned or explicitly superseded ADR-003 tooling. Address Dart async transport only through a demonstrated compatible approach; qualify every required install/runtime/device surface.
- [ ] 3. Establish and record acceptance A: Each required platform installs the packaged artifact and performs authenticated publish/watch/reconnect; supported offline CRDT behavior survives process replacement.
- [ ] 4. Establish and record acceptance B: Dart async remains blocked until a real transport proof passes; unsupported platforms remain excluded with explicit reasons, without claiming full phase closure or parity.
- [ ] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c022-platform-parity/`; run applicable local quality checks and `openspec validate pri-c022-platform-parity --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
