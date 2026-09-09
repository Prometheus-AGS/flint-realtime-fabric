# Tasks — p37-c004-aso-replica-runtime

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [x] T1: Worker DB owner with an exclusive-ownership lease; replace the ephemeral
      `new PGlite()` at `graph-provider.tsx:76-78`.
- [x] T2: Migration ledger recording applied migrations and the replica generation.
- [x] T3: Generic chunk→table writer taking table and columns as data — **no clinical table
      name appears in this change**.
- [x] T4: Publish one database revision as exactly one `ingestFetchedList` call
      (`graph.ts:360`); do not build a second publication path.
- [x] T5: Handle `must_refetch`: rebuild the affected generation and remove stale rows rather
      than merging. (This is G3's rebuild half.)
- [x] T6: Author a **subscriber-trace test** asserting a subscriber attached across a
      publication observes either the previous complete projection or the next one, never a
      partially updated relationship. ADR-009 requires traces, not rendered screens, because
      imperative subscribers observe Zustand directly and React batching does not cover them.
- [x] T7: Author tests against a **fixture schema**, not any ASO clinical table.
- [x] T8: Spec delta — `specs/replica-materialization/spec.md`, `## ADDED Requirements`.
      **P1 gate:** `openspec validate p37-c004-aso-replica-runtime` passes.

## Status 2026-09-07 — DONE, not archived

8/8; `openspec validate` passes; verify PASS. Typecheck clean across
`entity-graph-core`, `entity-graph-react`, `a2ui-react` and ASO `web/`.

Committed: ASO `e1622d9` (5 modules, 4 test files, 38 tests).

**G3 is now closed** — c003 delivered the commit half, this delivers the rebuild half.

**Not archived.** 38 tests authored and type-checking, none executed. The subscriber-trace
test in particular is the one ADR-009 requires be *run* before coherence may be claimed.
