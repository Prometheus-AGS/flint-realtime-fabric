# Tasks — p37-c001-pem-runtime-scoped-state

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Testing is deferred by operator directive, so this change
> can reach DONE but not PROVEN.

- [x] T1: Re-confirm the cited lines before editing — `local-first-runtime.ts:119` (module-scope
      `pendingActions`), `:164` (`clear()` in hydrate), `:208`/`:229`/`:263` (`isSynced`). They
      were verified 2026-09-06 and will drift.
- [x] T2: Move `pendingActions` into the runtime instance returned by `startLocalFirstGraph`;
      remove the module-scope binding.
- [x] T3: Make the hydrate-time clear operate on the instance's set, so a hydrating runtime
      cannot erase a sibling's pending actions.
- [x] T4: Compute `isSynced` from the owning runtime's set at all three sites.
- [x] T5: Scope `graphSyncStatusStore` per runtime; update `entity-graph-react`'s
      `useGraphSyncStatus` (`graph-store.ts:123`) in lockstep so the hook does not break.
- [x] T6: Compose the storage key from deployment + practice + identity + authorization-scope
      revision + replica generation (source-doc §7); update ASO `graph-provider.tsx:82`.
- [x] T7: Author a test in `local-first-runtime.test.ts` proving two runtimes in one process
      hold independent pending sets, and that runtime B hydrating leaves runtime A's pending
      actions intact.
- [x] T8: Author a test proving two identities in the same practice resolve to different
      storage keys.
- [x] T9: Write the spec delta — `specs/local-first-runtime/spec.md`, `## MODIFIED Requirements`
      with a MUST line and `#### Scenario:` blocks. **P1 gate:** `openspec validate
      p37-c001-pem-runtime-scoped-state` passes.
- [x] T10: Record the major-version implication for `@prometheus-ags/entity-graph-core` and
      `entity-graph-react` (both move together).

## Status 2026-09-06 — DONE, not archived

All 10 tasks complete; `openspec validate` passes; `tsc --noEmit` clean for
`entity-graph-core`, `entity-graph-react` and ASO `web/`.

Committed: PEM `97baec05`, ASO `76556bb`.

**Left in `openspec/changes/` rather than archived on purpose.** The four PEM regression tests
and six ASO storage-key tests are authored and compile but have **not been executed**, per the
no-testing directive. Archiving would read as certified. Archive it when the tests run.
