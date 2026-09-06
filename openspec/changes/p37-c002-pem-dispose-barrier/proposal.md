# p37-c002 — Async dispose barrier that drains in-flight persistence

## Summary

Disposal cannot currently stop a persist already in flight, and in ASO it cannot be called at
all. Two independent defects in two repos.

## Evidence

**PEM** — `packages/entity-graph-core/src/local-first-runtime.ts:216`:

```ts
      void persistGraphToStorage({ storage: opts.storage, key, store: storeApi });
```

`void` discards the promise. `dispose()` (`:270-275`) clears the debounce timer, which stops a
*scheduled* persist but leaves nothing to await for one already running.

**ASO** — `web/src/app/providers/graph-provider.tsx:82` calls `startLocalFirstGraph(...)` and
**never captures the return value**, so `dispose`/`persistNow`/`hydrate` are unreachable. The
effect teardown (`:96-99`) only sets `cancelled` and closes PGlite.

## The React constraint

A React effect cleanup is **synchronous** and cannot await a promise. "Await `dispose()` in the
teardown" would not produce a barrier. This change introduces an explicit session/lifecycle
manager that owns the runtime, records the disposal promise, and makes the *next* open await or
supersede it; the teardown calls into that manager rather than doing the waiting itself.

This is also what prevents StrictMode's double mount/unmount from creating two owners or letting
a late open resurrect a disposed session — the "late open resurrects a disposed session" hazard
named in source-doc §7.

## Scope

- `dispose()` becomes `async dispose(): Promise<void>`, awaiting or cancelling every tracked
  in-flight write (possible only because c001 gave the runtime instance state to track them in).
- Retain each persist promise in the runtime's in-flight set instead of discarding it.
- ASO gains a session/lifecycle manager that captures the runtime and sequences disposal
  against the next open.

## Files

| File | Repo | Change |
|---|---|---|
| `packages/entity-graph-core/src/local-first-runtime.ts` | PEM | in-flight set; async dispose |
| `packages/entity-graph-core/src/local-first-runtime.test.ts` | PEM | barrier tests |
| `web/src/app/providers/graph-provider.tsx` | ASO | capture runtime; call the manager |
| ASO session/lifecycle manager (new) | ASO | owns disposal ordering |
