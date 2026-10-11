# Tasks — pri-c012-sdk-pem-network

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect and record the clean/dirty baselines of Fabric, Forge, prometheus-entity-sync and PEM; verify the archived c004/c005 receipts and the implemented c011 tasks 2–3; identify the existing PGlite table/PEM transport, Fabric shape facade and Forge mutation endpoints to reuse. c011's deferred combined-campaign receipt is not an implementation blocker.
- [x] 2. Implement the complete online loop: optimistic PGlite mutation -> authenticated Forge write -> PostgreSQL commit -> Electric/Fabric authorized shape -> PGlite canonical reconciliation -> PEM entity/list update. Add a checked-in launch command and usable example UI/API interaction.
- [x] 3. Finish multi-client and lifecycle behavior in that same loop: create/update/delete, tenant/field projection, logout/owner switch, canonical replacement of optimistic state and explicit realtime-source fallback.
- [x] 4. Keep the checked-in prototype runnable while c013 adds durable outbox and restart behavior. Do not run a c012-only behavioral campaign or add isolated implementation-mirroring tests.
- [x] 5. After c013 tasks 2–3 are functional, run one combined local full-stack campaign from user mutation through second-client PEM observation, including the c011 GraphQL/rollback cases and c013 offline/restart cases. Bind c012 evidence to that receipt, then run build/typecheck/format/OpenSpec validation and resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
