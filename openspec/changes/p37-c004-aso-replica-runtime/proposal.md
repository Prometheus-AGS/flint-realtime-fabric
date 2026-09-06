# p37-c004 — Schema-agnostic ASO replica runtime

## Summary

Build the consumer that reads FRF's `GET /v1/shape` and publishes through PEM's existing atomic
surface — **without hardcoding any clinical table**. Every table is supplied by the caller from
the shape catalog, so nothing here presumes a schema ASO has not approved.

## Why schema-agnostic

ASO's privacy-approved replica schema is sequence step 1 and does not exist. An earlier draft of
this plan scheduled concrete clinical materialization here anyway, which adversarial review
correctly flagged: it would put unapproved column names into a migration ledger and guarantee
the "provisional, must be revisited" churn the assessment warned about. Concrete tables are
deferred to c006; this change proceeds now because a generic runtime needs no approved schema.

## PEM's atomic surface already exists — do not build another

`packages/entity-graph-core/src/graph.ts:360`, `ingestFetchedList`, is a single `set(...)`
committing the primary entity batch, `options.sideBatches` ("rows that must commit or fail with
the primary batch"), `lists`, view-backed `projections` and `finishListFetches` in one Zustand
publication. ADR-009 hedged that this might need building; it does not.

## Carries G3's rebuild half

`must_refetch` — discard the affected replica generation and **remove stale rows** rather than
merging a new snapshot into old data — lands here, because it needs a chunk consumer. FRF
already surfaces the signal (`ShapeChunk.must_refetch`; gateway maps it to HTTP 409 plus
`electric-must-refetch`). This change is its first consumer, and closes G3 together with c003.

## Scope

- Worker DB owner with an exclusive-ownership lease and a migration ledger, replacing the bare
  `new PGlite()` at `graph-provider.tsx:76-78` that its own comment marks ephemeral.
- A **generic** chunk→table writer driven by the shape's declared table and columns.
- One database revision publishes as one `ingestFetchedList` call.
- `must_refetch` rebuild-and-remove-stale-rows handling.

## Files

| File | Repo | Change |
|---|---|---|
| ASO replica runtime (new) | ASO | worker DB owner, lease, ledger, generic writer |
| `web/src/app/providers/graph-provider.tsx` | ASO | use the owned DB, not a bare PGlite |
| `crates/frf-shape-electric/**` | FRF | only if the chunk contract needs adjusting |
