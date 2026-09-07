# p37-c006 — Conform the FRF shape catalog to ASO's existing replica schema

## Status: UNBLOCKED (2026-09-06) — the premise was wrong

This change was filed as BLOCKED on "ASO's privacy-approved replica schema does not exist
(sequence step 1)." **That was incorrect.** The schema exists, is tested, and was designed for
this data path. The block came from reading the ASO runtime architecture's sequence table —
which lists the schema as step 1 — and inferring it had not been done, without checking the ASO
repo. It had been done.

The change is therefore re-scoped from *"define clinical tables"* to *"conform FRF's catalog to
the tables ASO already defined."* The conformance direction runs toward ASO, not away from it.

## What ASO already has

`web/src/shared/sync/pglite-schema.ts` (repo: `prior-auth`):

- Five tables and nothing else: `cases`, `case_evidence`, `evidence_states`,
  `evidence_citations`, `documents`.
- `OMITTED_COLUMNS` records every exclusion **as data with a reason** — `rationale`, `quote`,
  `patient_id`, `author_name`, `author_npi`, `storage_uri`, `data`.
- `pglite-schema.test.ts` fails on a sixth table, on a PHI-suggestive table name, or on an
  omitted column reappearing.
- ADR-007 backs it; enforcement is build-time plus that test, stated plainly.

`web/src/shared/sync/electric-shapes.ts`:

- `SYNC_RELATIONS` — base tables (`aso.cases`, …), not views. Measured 2026-09-05: a view
  returns 400 from `GET /v1/shape` because it emits no WAL and cannot join a publication.
- `SYNC_COLUMNS` — **is** the PHI boundary on the wire, verified against a canary row.
- `createTenantScopedElectricAdapter` fails closed on a table with no tenant decision.
- `practice_id` is denormalized onto every row *because an Electric shape WHERE clause is flat
  and cannot join* — the schema already anticipates the facade's request model.

## Scope

Produce an FRF shape catalog whose entries mirror ASO's `SYNC_RELATIONS` and `SYNC_COLUMNS`
exactly, with `practice_id` as the scope column and `view` on `practice` as the Keto relation.

**No ASO schema change.** Widening `SYNC_COLUMNS` or adding a table to suit the facade would
invert the direction of conformance and weaken a boundary that has a test behind it —
`OMITTED_COLUMNS` states that removing an entry "is a decision about PHI, not a cleanup."

## Prerequisite that is genuinely open

Whether ASO adopts the facade at all. ASO syncs Electric **directly** today
(`Postgres → Electric shapes → PGlite`), and that path works and is measured. Inserting FRF as
an authorizing proxy is an architecture decision with a live alternative, and it determines
whether this change ships or is withdrawn. Recorded in
`docs/architecture/frf-shape-facade-integration.md` in the `prior-auth` repo.

## Files

| File | Repo | Change |
|---|---|---|
| shape catalog JSON | FRF | entries mirroring ASO's five tables |
| `docs/SHAPE-FACADE.md` | FRF | replace the illustrative example with the real catalog |
| `docs/architecture/frf-shape-facade-integration.md` | ASO | the integration note (written) |
