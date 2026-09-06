# p37-c003 — Commit replica rows and resume checkpoint atomically

## Summary

There is no transaction boundary and no checkpoint surface in PEM's persistence adapter, and the
Electric adapter throws the resume position away. Rows and checkpoint cannot describe the same
commit boundary because the checkpoint does not exist.

## Evidence

`packages/entity-graph-core/src/adapters/pglite-persistence.ts` (102 lines) exports only
`PGlitePersistenceClient`, `CreatePGlitePersistenceAdapterOptions` and
`createPGlitePersistenceAdapter` — no transaction, no checkpoint.

`packages/entity-graph-core/src/adapters/electricsql.ts:90` builds each change with
`offset: ""`. An empty offset cannot describe a commit boundary.

## G3 is split across two changes

This change owns G3's **commit** half. G3's **rebuild** half — `must_refetch` discarding a
generation and removing stale rows — belongs to c004, because it needs a consumer that fetches
chunks and none exists until then. **c003 does not close G3.**

## Scope

- A transactional write in the PGlite adapter committing rows **and** checkpoint together.
- Carry Electric's real offset instead of `""`.
- On resume, validate that persisted rows and checkpoint describe the same boundary; a mismatch
  rebuilds rather than merging.

## Files

| File | Repo | Change |
|---|---|---|
| `packages/entity-graph-core/src/adapters/pglite-persistence.ts` | PEM | transaction + checkpoint |
| `packages/entity-graph-core/src/adapters/electricsql.ts` | PEM | carry the real offset |
| `packages/entity-graph-core/src/adapters/pglite-persistence.test.ts` | PEM | atomicity tests |
