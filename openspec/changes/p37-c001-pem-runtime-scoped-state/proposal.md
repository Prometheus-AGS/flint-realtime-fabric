# p37-c001 — Scope PEM runtime state per runtime; compose the storage key

## Summary

`pendingActions` lives at module scope in PEM's `local-first-runtime.ts`, so every runtime in a
process shares one map. This is not merely shared state: `pendingActions.clear()` inside hydrate
means a second runtime hydrating **erases the first runtime's un-settled pending actions** —
silent clinical-write loss on an account or practice switch.

Move pending actions and sync status into the runtime instance, and compose the persistence key
from more than practice.

## Evidence

`packages/entity-graph-core/src/local-first-runtime.ts` (repo:
`../prometheus-entity-management`), verified 2026-09-06:

- `:119` — `const pendingActions = new Map<string, GraphActionRecord>();` (module scope)
- `:164` — `pendingActions.clear();` (inside hydrate)
- `:208`, `:229`, `:263` — `isSynced: pendingActions.size === 0` reads the shared map
- `:167`, `:227` — `graphSyncStatusStore` is likewise a module singleton

ASO, `web/src/app/providers/graph-provider.tsx:82`:

```ts
startLocalFirstGraph({ storage, store, key: `aso:${session.practiceId}` });
```

Source-doc §7 (`prior-auth` repo, `application-runtime-architecture.md`, commit `6bf36c2`):
"The persisted namespace is based on deployment, practice, identity, authorization-scope
revision and replica generation… **Do not key private storage only by practice ID.**"

## Blast radius — this is a breaking public API change

`pendingActions`, `graphSyncStatusStore` and `startLocalFirstGraph` are all re-exported from
`@prometheus-ags/entity-graph-core` (4.0.0) and again from `entity-graph-react`, whose
`useGraphSyncStatus` hook (`entity-graph-react/src/graph-store.ts:123`) reads the singleton
directly. Both packages move together; the version bump is **major**.

## Scope

- Own `pendingActions` per runtime instance created by `startLocalFirstGraph`.
- Make the hydrate-time clear an instance operation.
- Compute `isSynced` from the owning runtime's set.
- Scope `graphSyncStatusStore` per runtime; update `useGraphSyncStatus` in lockstep.
- Compose the storage key from deployment + practice + identity + authorization-scope revision +
  replica generation.

## Non-goals

The dispose barrier (c002) and checkpoint atomicity (c003). This change only relocates
ownership, which is what makes those possible.

## Files

| File | Repo | Change |
|---|---|---|
| `packages/entity-graph-core/src/local-first-runtime.ts` | PEM | pending actions + status per runtime |
| `packages/entity-graph-react/src/graph-store.ts` | PEM | `useGraphSyncStatus` follows the scoping |
| `packages/entity-graph-core/src/local-first-runtime.test.ts` | PEM | regression tests |
| `web/src/app/providers/graph-provider.tsx` | ASO | composed storage key |
