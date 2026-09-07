# Tasks — p37-c006-aso-clinical-materialization

> **UNBLOCKED 2026-09-06.** Was filed as blocked on a schema that already existed; see the
> proposal. Re-scoped to conformance: FRF's catalog follows ASO's tables, never the reverse.
>
> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [ ] T1: Re-read ASO's `SYNC_RELATIONS` and `SYNC_COLUMNS` (`web/src/shared/sync/electric-shapes.ts`)
      and `PGLITE_TABLES` / `OMITTED_COLUMNS` (`pglite-schema.ts`). They are the source of truth;
      copy from them rather than restating them from memory.
- [ ] T2: Write the FRF shape catalog with one entry per ASO table, `table` set to the base
      relation (`aso.cases`, …) — **not** a view, which returns 400 from Electric.
- [ ] T3: Set each entry's `columns` to exactly `SYNC_COLUMNS[table]`. Adding a column here
      widens the PHI boundary and must not happen without an ASO decision.
- [ ] T4: Set `relation: "view"` and `object_namespace: "practice"`; the scope column is
      `practice_id`, already denormalized onto every synced row.
- [ ] T5: Author a **drift test** asserting the FRF catalog and ASO's `SYNC_COLUMNS` agree
      column-for-column. Two independently-maintained copies of a PHI boundary will drift; this
      is what catches it.
- [ ] T6: Replace the illustrative catalog in `docs/SHAPE-FACADE.md` with the real one, and
      delete the "illustrative only, do not deploy" warning that no longer applies.
- [ ] T7: Spec delta — `specs/replica-materialization/spec.md`, `## MODIFIED Requirements`.
      **P1 gate:** `openspec validate p37-c006-aso-clinical-materialization` passes.

## Not a task here

Deciding whether ASO adopts the facade at all. ASO syncs Electric directly today and that path
works; inserting FRF as an authorizing proxy is an architecture decision, not an implementation
detail. If the answer is no, this change is withdrawn rather than completed.
