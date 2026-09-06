# Tasks — p37-c004-aso-replica-runtime

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [ ] T1: Worker DB owner with an exclusive-ownership lease; replace the ephemeral
      `new PGlite()` at `graph-provider.tsx:76-78`.
- [ ] T2: Migration ledger recording applied migrations and the replica generation.
- [ ] T3: Generic chunk→table writer taking table and columns as data — **no clinical table
      name appears in this change**.
- [ ] T4: Publish one database revision as exactly one `ingestFetchedList` call
      (`graph.ts:360`); do not build a second publication path.
- [ ] T5: Handle `must_refetch`: rebuild the affected generation and remove stale rows rather
      than merging. (This is G3's rebuild half.)
- [ ] T6: Author a **subscriber-trace test** asserting a subscriber attached across a
      publication observes either the previous complete projection or the next one, never a
      partially updated relationship. ADR-009 requires traces, not rendered screens, because
      imperative subscribers observe Zustand directly and React batching does not cover them.
- [ ] T7: Author tests against a **fixture schema**, not any ASO clinical table.
- [ ] T8: Spec delta — `specs/replica-materialization/spec.md`, `## ADDED Requirements`.
      **P1 gate:** `openspec validate p37-c004-aso-replica-runtime` passes.
