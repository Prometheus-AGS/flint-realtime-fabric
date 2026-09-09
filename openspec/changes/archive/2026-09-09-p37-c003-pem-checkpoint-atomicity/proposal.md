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

## G3 is not closed by this change — implemented 2026-09-06

c003 delivers G3's **commit** half: rows and checkpoint land in one statement,
and `evaluateResume` refuses a resume whose checkpoint does not describe the
stored rows.

G3's **rebuild** half — Electric's `must_refetch` discarding a generation and
removing stale rows — is carried by **c004**, because it needs a consumer that
fetches chunks and none exists until then. The pieces c003 leaves in place for it:

- `ReplicaCheckpoint.generation`, which c004 increments on a rebuild.
- `evaluateResume`'s `stale-generation` and `handle-changed` rejections, which
  are exactly the conditions a `must_refetch` produces.

**Do not mark G3 satisfied on c003 alone.** A replica that commits atomically but
never rebuilds will merge a fresh snapshot into stale rows the first time
Electric asks it to refetch — the failure ADR-009 names explicitly.

## A finding that changed the task

T4 was written as "stop sending `offset: \"\"`". The real defect was larger and
elsewhere: `toChange` read `msg.offset` from every shape message and **discarded
it**, so no offset reached a consumer by any path. `EntityChange` had no field to
carry one.

The fix puts the cursor on **`ChangeSet`**, not `EntityChange`: a resume position
describes a boundary *between* batches, not a point inside one, so it is taken
from the last message of a batch and attached once.

The `offset: ""` in the Postgres LISTEN/NOTIFY path is now documented as correct
rather than removed. A local trigger notification is not an Electric shape
message and genuinely has no offset; it emits no cursor, so that batch cannot be
checkpointed — which is the truthful outcome rather than a fabricated position.
