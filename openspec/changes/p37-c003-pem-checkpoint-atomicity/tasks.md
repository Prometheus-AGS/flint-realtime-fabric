# Tasks — p37-c003-pem-checkpoint-atomicity

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [ ] T1: Re-confirm `pglite-persistence.ts` has no transaction/checkpoint surface and
      `electricsql.ts:90` still sends `offset: ""`.
- [ ] T2: Add a checkpoint record (shape handle + offset + replica generation) to the adapter.
- [ ] T3: Commit rows and checkpoint in one transaction.
- [ ] T4: Carry Electric's real offset through `toChange` instead of `""`.
- [ ] T5: On resume, validate rows and checkpoint describe the same boundary; rebuild on
      mismatch rather than merging.
- [ ] T6: Author a test simulating an interruption between row write and checkpoint write,
      asserting the replica remains resumable with no torn state.
- [ ] T7: Spec delta — `specs/replica-persistence/spec.md`, `## ADDED Requirements`.
      **P1 gate:** `openspec validate p37-c003-pem-checkpoint-atomicity` passes.
- [ ] T8: Record in the change that G3's `must_refetch` half is carried by c004, so c003 is not
      mistaken for G3 complete.
