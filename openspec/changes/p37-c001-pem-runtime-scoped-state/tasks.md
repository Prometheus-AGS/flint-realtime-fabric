# Tasks — p37-c001-pem-runtime-scoped-state

> **DONE** = implemented, spec delta valid, tests authored and type-checking.
> **PROVEN** = those tests executed. Testing is deferred by operator directive, so this change
> can reach DONE but not PROVEN.

- [ ] T1: Re-confirm the cited lines before editing — `local-first-runtime.ts:119` (module-scope
      `pendingActions`), `:164` (`clear()` in hydrate), `:208`/`:229`/`:263` (`isSynced`). They
      were verified 2026-09-06 and will drift.
- [ ] T2: Move `pendingActions` into the runtime instance returned by `startLocalFirstGraph`;
      remove the module-scope binding.
- [ ] T3: Make the hydrate-time clear operate on the instance's set, so a hydrating runtime
      cannot erase a sibling's pending actions.
- [ ] T4: Compute `isSynced` from the owning runtime's set at all three sites.
- [ ] T5: Scope `graphSyncStatusStore` per runtime; update `entity-graph-react`'s
      `useGraphSyncStatus` (`graph-store.ts:123`) in lockstep so the hook does not break.
- [ ] T6: Compose the storage key from deployment + practice + identity + authorization-scope
      revision + replica generation (source-doc §7); update ASO `graph-provider.tsx:82`.
- [ ] T7: Author a test in `local-first-runtime.test.ts` proving two runtimes in one process
      hold independent pending sets, and that runtime B hydrating leaves runtime A's pending
      actions intact.
- [ ] T8: Author a test proving two identities in the same practice resolve to different
      storage keys.
- [ ] T9: Write the spec delta — `specs/local-first-runtime/spec.md`, `## MODIFIED Requirements`
      with a MUST line and `#### Scenario:` blocks. **P1 gate:** `openspec validate
      p37-c001-pem-runtime-scoped-state` passes.
- [ ] T10: Record the major-version implication for `@prometheus-ags/entity-graph-core` and
      `entity-graph-react` (both move together).
