# Tasks — p37-c005-evidence-not-loaded-state

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [ ] T1: Re-confirm `countStates` still seeds `{met: 0, gap: 0, void: 0}` and that no
      not-loaded representation has appeared since 2026-09-06.
- [ ] T2: Choose and implement the approach — an explicit not-loaded state on `EvidenceCounts`,
      or a load-state gate before counting. Record which and why.
- [ ] T3: Ensure `case-summary.ts` cannot derive clinical text from unloaded counts.
- [ ] T4: Author a test proving a mid-hydration timeline is not reported as a loaded all-zero
      case.
- [ ] T5: Author a test proving no clinical summary string is produced from unloaded counts.
- [ ] T6: Spec delta — `specs/evidence-state/spec.md`, `## MODIFIED Requirements`.
      **P1 gate:** `openspec validate p37-c005-evidence-not-loaded-state` passes.
