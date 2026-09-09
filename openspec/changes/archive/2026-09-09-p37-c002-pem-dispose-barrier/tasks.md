# Tasks — p37-c002-pem-dispose-barrier

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Deferred by operator directive.

- [x] T1: Re-confirm `local-first-runtime.ts:216` (`void persistGraphToStorage`) and
      `graph-provider.tsx:82` (uncaptured return) before editing.
- [x] T2: Track each persist promise in the runtime's in-flight set rather than discarding it.
- [x] T3: Make `dispose()` async, awaiting or cancelling every tracked write; keep clearing the
      debounce timer for scheduled-but-unstarted work.
- [x] T4: Build the ASO session/lifecycle manager: capture the runtime, hold the disposal
      promise, and make the next open await or supersede it. **Do not** await inside the React
      cleanup — it is synchronous and cannot.
- [x] T5: Wire `graph-provider.tsx` teardown to the manager.
- [x] T6: Author a test proving no persistence write lands after the disposal promise resolves,
      including a write started immediately before disposal.
- [x] T7: Author a test proving a StrictMode-style double mount/unmount yields exactly one
      owner and no resurrected session.
- [x] T8: Spec delta — `specs/local-first-runtime/spec.md`, `## MODIFIED Requirements`.
      **P1 gate:** `openspec validate p37-c002-pem-dispose-barrier` passes.

## Status 2026-09-06 — DONE, not archived

8/8 tasks; `openspec validate` passes; verify PASS. `tsc --noEmit` clean for
`entity-graph-core`, `entity-graph-react`, `a2ui-react` and ASO `web/`.

Committed: PEM `4614976b`, ASO `eb03edd`.

**Not archived, deliberately.** The four PEM barrier tests and eight ASO manager tests are
authored and type-check but have **not run**. Archiving would read as certified. Archive when
the tests execute.
