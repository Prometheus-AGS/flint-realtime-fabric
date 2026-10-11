# Tasks — pri-c013-sync-persistence

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Once c012 tasks 2–3 are functional, record its implementation baseline and define the PGlite outbox schema, stable mutation identity, Forge idempotency/version boundary and canonical acknowledgement rule. c012's deferred combined-campaign receipt is not an implementation blocker.
- [x] 2. Implement the complete durable flow: persist optimistic mutations and retry state in PGlite, dispatch through Forge after reconnect/restart, apply stable idempotency at the server boundary, and retain entries until Electric/Fabric returns canonical state.
- [x] 3. Finish conflict and multi-client behavior: expose rejection/version conflict, support recoverable retry or user resolution, reconcile create/update/delete, and converge two clients without violating tenant isolation.
- [x] 4. Once tasks 2–3 are functional, run the single combined local full-stack campaign for c011–c013 through PGlite, Forge, PostgreSQL, Electric/Fabric, Forge GraphQL and PEM. Include offline mutation, client/Forge restart, retry/idempotency, conflict visibility, two-client convergence, authorization, checkpoint reconnect and LISTEN rollback. Isolated storage or transport tests do not satisfy acceptance.
- [x] 5. Save one source-bound full-flow receipt and reference it from c011, c012 and c013; run their build/typecheck/format/OpenSpec validation and resolve required review findings before archiving all three.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
