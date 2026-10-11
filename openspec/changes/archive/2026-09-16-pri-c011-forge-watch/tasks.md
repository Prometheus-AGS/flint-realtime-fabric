# Tasks — pri-c011-forge-watch

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c005-authority-lifetime, pri-c010-type-watch.
- [x] 2. Finish the complete runnable lane: use the c010 tonic client and canonical mapping to deliver SQL commits through Fabric WatchEntityType into authenticated Forge GraphQL subscriptions, including RLS projection, safe delete/invalidation behavior and truncation reconstruction. Keep LISTEN selectable as the rollback source.
- [x] 3. Complete functional recovery behavior in the same lane: reconnect from checkpoints, expose lag/resync and backend failures, enforce revocation and preserve an explicit LISTEN/Fabric switch without silent loss or double application.
- [x] 4. After c012 and c013 are functional, run one combined local end-to-end campaign covering this GraphQL lane plus the complete PGlite -> Forge -> PostgreSQL -> Electric/Fabric -> PGlite/PEM flow. The c011 portion must cover authenticated create/update/delete, denied rows, composite/key changes, checkpoint reconnect, truncated LISTEN reconstruction and Fabric/LISTEN rollback. Do not interrupt implementation with a smaller c011-only campaign.
- [x] 5. Bind c011 evidence to that combined campaign, then run build/format/OpenSpec validation and resolve required review findings before archive. Keep c011 open until the combined receipt exists; its open evidence tasks do not block implementation of c012 or c013.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
