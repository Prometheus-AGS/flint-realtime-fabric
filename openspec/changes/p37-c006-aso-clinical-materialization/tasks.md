# Tasks — p37-c006-aso-clinical-materialization

> **BLOCKED — do not start.** Requires ASO's privacy-approved replica schema (sequence step 1),
> which does not exist. Tracked so the dependency is visible.

- [ ] T0: **Gate** — confirm ASO has published the privacy-approved replica schema. If not,
      STOP; this change stays unstarted.
- [ ] T1: Migrations for the approved clinical tables, recorded in c004's migration ledger.
- [ ] T2: Typed repositories over those tables.
- [ ] T3: Shape catalog entries naming them, consumed by c004's generic writer.
- [ ] T4: Author tests over the real schema, replacing c004's fixture-schema coverage.
- [ ] T5: Spec delta — `specs/replica-materialization/spec.md`, `## MODIFIED Requirements`.
      **P1 gate:** `openspec validate p37-c006-aso-clinical-materialization` passes.
