# Focused review: combined c011-c013 candidate

Reviewed: 2026-09-17

Verdict: **PASS**

Required findings: none.

## Scope reviewed

- Forge GraphQL dynamic value projection, PostgreSQL native JSON projection,
  primary/foreign-key reflection, fail-closed no-PK behavior, idempotency, and
  mutation conflict mapping.
- Prototype Compose topology, fixture schema, composite keys, RLS, publication
  enrollment, and explicit LISTEN trigger setup.
- PGlite canonical shadow table, optimistic application table, durable outbox,
  canonical acknowledgement, reconnect reconciliation, numeric normalization,
  and startup behavior while Electric catches up.

## Review conclusions

- Authorization stays at Forge's Cedar and PostgreSQL forced-RLS boundaries;
  GraphQL emits the RLS re-query result rather than the raw change payload.
- Tables without a reflected primary key fail closed for subscription
  re-query/delete reconstruction.
- Unique-constraint details remain server-side while callers receive a generic
  recoverable `409`.
- Stable idempotency identities prevent duplicate writes; a changed-body replay
  conflicts and the browser keeps rejected entries visible for resolution.
- Canonical replication uses a PGlite shadow table, avoiding initial-sync
  insert collisions with optimistic rows. Outstanding mutations are retained
  until matching canonical state arrives.
- All touched hand-authored source files remain below 500 lines. No dependency
  direction or frozen-protobuf contract change was introduced.

The remaining PGlite IndexedDB reopen latency is recorded in the receipt. It
does not cause local or acknowledged state loss and does not block these three
acceptance contracts.
