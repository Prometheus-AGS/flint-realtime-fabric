# p37-c006 — Concrete clinical materialization (BLOCKED)

## Status: BLOCKED — do not start

ASO's privacy-approved replica schema is **sequence step 1** of the ASO runtime architecture and
does not exist. This change binds c004's generic runtime to real clinical tables — migrations,
typed repositories, and the shape catalog entries naming them.

Starting it without the approved schema would write unapproved clinical column names into a
migration ledger and guarantee rework. `docs/examples/shape-catalog.example.json` is
**illustrative only** and is not approved schema.

## Unblocks when

ASO publishes the privacy-approved replica schema and its shape catalog.

## Scope (when unblocked)

- Migrations for the approved clinical tables, recorded in c004's ledger.
- Typed repositories over those tables.
- Shape catalog entries naming them, consumed by c004's generic writer.

## Why it is in the plan at all

So the dependency is visible and tracked rather than rediscovered later. It is expected to
remain unstarted for the duration of this phase, and the phase's deliverable count (5) excludes
it.
