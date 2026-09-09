# p37-c005 — Distinguish "not loaded" from a loaded empty case

## Summary

`met`/`gap`/`void` has no not-loaded state, so a mid-hydration timeline is structurally
identical to a fully-loaded case with no entries — and that ambiguity feeds clinical text.

## Evidence

`web/src/features/evidence-timeline/components/evidence-timeline.tsx:113-119`:

```ts
export function countStates(entries: readonly TimelineEntry[]): EvidenceCounts {
  const counts: EvidenceCounts = { met: 0, gap: 0, void: 0 };
  for (const entry of entries) {
    counts[entry.state] += 1;
  }
  return counts;
}
```

`EvidenceCounts` carries exactly three states; a repo-wide search for `notLoaded` / `not_loaded`
in `web/src` returns nothing. The zero seeding is documented at `:110-112` as deliberate so "a
state with no entries reports `0` rather than being absent" — correct for a *loaded* case,
precisely wrong for an unloaded one.

Downstream, `features/case-queue/model/case-summary.ts:20` turns these counts into clinical
text: `{met: 4, gap: 3, void: 1}` should read "1 document to obtain." A partially hydrated case
therefore renders a confident clinical summary from rows that have not arrived.

## Why this is independent

The defect lives in the evidence state model, not the replication path. It proceeds in parallel
with c001–c004 and must not be sequenced behind the materializer it will eventually protect.

## Scope

Either add an explicit not-loaded representation to `EvidenceCounts`, or gate callers on load
state before counting. Either satisfies G4a; the choice is the implementer's.

## Files

| File | Repo | Change |
|---|---|---|
| `web/src/shared/model/evidence-state.ts` | ASO | not-loaded representation |
| `web/src/features/evidence-timeline/components/evidence-timeline.tsx` | ASO | `countStates` |
| `web/src/features/case-queue/model/case-summary.ts` | ASO | no summary from unloaded counts |
