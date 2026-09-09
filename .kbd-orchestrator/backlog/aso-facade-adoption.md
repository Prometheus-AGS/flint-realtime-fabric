# Carried forward — ASO shape-facade adoption

> Withdrawn from phase-36 as `p37-c006-aso-clinical-materialization` on 2026-09-09,
> at 0/7 tasks with no spec delta. Not abandoned — **blocked on a decision that is
> not an implementation detail**, and kept here so the next phase inherits the
> context rather than rediscovering it.

## The open question

**Does ASO adopt the FRF authorized shape facade, or not?**

Everything below is downstream of that answer. The change should not be
implemented before it, and if the answer is no it is withdrawn rather than
completed — that was the original change's own "Not a task here" clause.

## Why it is genuinely open

**The case for adopting.** ADR-009 states the position plainly:

> A client predicate or tenant-scoped adapter is useful validation but cannot
> enforce access against a modified client.

ASO's own ADR-007 — the column-exclusion decision — now carries a superseding
banner saying the same thing: projection privacy requires server enforcement.
By that reasoning ASO's current client-side boundary is validation, not a
control, and the facade is what makes it a control.

**The case against, or at least for waiting.** ASO syncs ElectricSQL directly
today, that path works, and it is measured. The FRF facade is built, **disabled
behind an off-by-default `shape-facade` Cargo feature, and has never been run
against a live Electric server**. Adopting it means replacing a working measured
path with an unproven one. The gateway itself logs a warning saying the lane is
not certified.

So this is not "finish the implementation" — it is a choice between a working
path with a weaker boundary and an unproven path with a stronger one.

## If the answer is YES — the work, as scoped

Conformance runs **toward ASO**: FRF's catalog follows ASO's tables, never the
reverse.

1. Re-read ASO's `SYNC_RELATIONS` / `SYNC_COLUMNS` (`web/src/shared/sync/electric-shapes.ts`)
   and `PGLITE_TABLES` / `OMITTED_COLUMNS` (`pglite-schema.ts`). They are the
   source of truth — copy from them, do not restate from memory.
2. Write the FRF shape catalog with one entry per ASO table, `table` set to the
   **base relation** (`aso.cases`, …). A view returns 400 from Electric — it
   emits no WAL of its own, so it can never join a publication.
3. Set each entry's `columns` to exactly `SYNC_COLUMNS[table]`. **Adding a column
   here widens the PHI boundary** and must not happen without an ASO decision.
4. Set `relation: "view"`, `object_namespace: "practice"`; the scope column is
   `practice_id`, already denormalized onto every synced row and forced by a
   server-side trigger, so a caller cannot set it or lie about it.
5. Author a **drift test** asserting the FRF catalog and ASO's `SYNC_COLUMNS`
   agree column-for-column. Two independently maintained copies of a PHI
   boundary will drift; this is what catches it.
6. Replace the illustrative catalog in `docs/SHAPE-FACADE.md` with the real one
   and delete the "illustrative only, do not deploy" warning.
7. Spec delta against `specs/replica-materialization/`. Note the capability
   **already exists**, so `## MODIFIED Requirements` is correct here — unlike
   c001/c002/c005, which had to be corrected to `ADDED` because their
   capabilities were new.

## If the answer is NO

Withdraw permanently. Record the decision in an ADR so the next reader does not
re-litigate it, and delete the illustrative catalog from `docs/SHAPE-FACADE.md`
rather than leaving it to look like a plan.

## A correction worth keeping

This change was originally filed as BLOCKED on "ASO's privacy-approved replica
schema does not exist." **That was wrong.** The schema exists, is tested, and was
designed for this data path. The error came from reading the ASO runtime
architecture's sequence table — which lists the schema as step 1 — and inferring
it had not been done, without checking the repo. It had been done.

The lesson generalises: *listed as a future step* is not evidence something has
not happened. Check the repo.
