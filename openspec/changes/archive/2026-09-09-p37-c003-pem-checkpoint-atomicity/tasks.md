# Tasks — p37-c003-pem-checkpoint-atomicity

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [x] T1: Re-confirm `pglite-persistence.ts` has no transaction/checkpoint surface and
      `electricsql.ts:90` still sends `offset: ""`.
- [x] T2: Add a checkpoint record (shape handle + offset + replica generation) to the adapter.
- [x] T3: Commit rows and checkpoint in one transaction.
- [x] T4: Carry Electric's real offset through `toChange` instead of `""`.
- [x] T5: On resume, validate rows and checkpoint describe the same boundary; rebuild on
      mismatch rather than merging.
- [x] T6: Author a test simulating an interruption between row write and checkpoint write,
      asserting the replica remains resumable with no torn state.
- [x] T7: Spec delta — `specs/replica-persistence/spec.md`, `## ADDED Requirements`.
      **P1 gate:** `openspec validate p37-c003-pem-checkpoint-atomicity` passes.
- [x] T8: Record in the change that G3's `must_refetch` half is carried by c004, so c003 is not
      mistaken for G3 complete.

## Status 2026-09-06 — DONE, not archived

8/8; `openspec validate` passes; verify PASS. Typecheck clean: `entity-graph-core`,
`entity-graph-react`, `a2ui-react`, ASO `web/`.

Committed: PEM `297cccea`.

**Not archived.** Eleven tests authored and type-checking, none executed. Archiving would read
as certified. Also note G3 stays open until c004 lands its rebuild half.
