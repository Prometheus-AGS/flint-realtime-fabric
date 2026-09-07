# Assessment — adr-009-replication-materializer-and-pem-publication

> Child of `phase-36-sovereign-sfu-ice-linux-fix`. Assessed 2026-09-06 against the child's
> four goals. Backend: OpenSpec. Repos in scope: PEM, ASO web, FRF (shape-facade surface only).

## Evidence provenance — read this before the findings

Every code claim in this assessment cites a file in **another repository**:

| Repo | Absolute root |
|---|---|
| PEM | `/Users/gqadonis/Projects/prometheus/prometheus-entity-management` |
| ASO web | `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web` |

The adversarial-review packet builder resolves cited paths against **this** repo only, so its
`cited_paths` block reported all nine as `MISSING` and the first review round returned a
CRITICAL: the claims could not be checked from the packet. That was a fair call on the evidence
supplied.

The fix is not to assert harder. **Every load-bearing claim below now quotes the source line
verbatim**, so the assertion and its evidence travel together and a reader can verify without
either repo checked out. Each quote was re-read from disk at assessment time
(`sed -n '<line>p' <file>`); line numbers are as of 2026-09-06.

Claims that could *not* be substantiated this way are marked **UNVERIFIED** rather than stated
as fact.

## Summary

The FRF read path is done (commit `83d7c23`). The replica side is **largely unbuilt**, and two
defects are materially worse than the child's seeded goals stated. One goal is cheaper than
expected because the surface it needs is already present.

Two findings below (**G4a** and the **storage key**) were not in the child's four seeded goals.
G4a came out of adversarial review — the reviewer noticed G4's "not loaded" clause had gone
unassessed, and checking it found a real defect. Both are proposed additions to the phase, not
already-agreed scope.

| Goal | State | Severity |
|---|---|---|
| G1 — per-runtime state | NOT MET — worse than seeded: cross-runtime data loss, not just sharing | **CRITICAL** |
| G2 — async dispose barrier | NOT MET — worse than seeded: the handle is never captured in ASO, so `dispose()` is unreachable | **CRITICAL** |
| G3 — atomic rows + checkpoint | NOT MET — no transaction or checkpoint surface exists at all | **HIGH** |
| G4 — materializer + coherent publication | PARTIAL — a candidate atomic surface exists (coherence unverified); no materializer consumes it | **HIGH** |
| G4a — `met`/`gap`/`void` vs "not loaded" | NOT MET — no not-loaded state exists; mid-hydration is indistinguishable from a loaded empty case | **HIGH** |
| (proposed) storage key scoped by practice alone | NOT MET — violates §7 of the source architecture | **HIGH** |

## G1 — Per-runtime state: NOT MET (critical)

`packages/entity-graph-core/src/local-first-runtime.ts:119`

```ts
const pendingActions = new Map<string, GraphActionRecord>();
```

Module scope, shared by every runtime in the process. The seeded goal described this as "two
runtimes share it." The actual behaviour is **destructive**:

- **`:164` — `pendingActions.clear()` inside hydrate**, verbatim:
  ```ts
      pendingActions.clear();
  ```
  A second runtime hydrating *erases the first runtime's un-settled pending actions*. This is
  silent clinical-write loss on an account or practice switch, not merely shared state.
- **`:208`, `:229`, `:263` — `isSynced: pendingActions.size === 0`.** Sync status is computed
  from the shared map, so runtime A reports "synced" while runtime B still has writes pending,
  or vice versa.

`graphSyncStatusStore` (`:167`, `:227`) is likewise a module singleton, so status published by
one runtime overwrites the other's.

**Implication for the phase:** this must be fixed before any materializer work, because a
materializer that publishes into a graph whose pending set can be wiped by a sibling runtime
cannot be made correct downstream.

## G2 — Async dispose barrier: NOT MET (critical)

Two independent defects, in different repos.

**PEM — the persist promise is discarded.** `local-first-runtime.ts:213-217`:

```ts
persistTimer = setTimeout(() => {
  void persistGraphToStorage({ storage: opts.storage, key, store: storeApi });
}, persistDebounceMs);
```

`void` throws the promise away. `dispose()` (`:270-275`) clears the timer, which stops a
*scheduled* persist but cannot stop or await one already **in flight**. There is no handle to
await, so no barrier is constructible without changing this call site.

**ASO — `dispose()` is unreachable.** `web/src/app/providers/graph-provider.tsx:82`:

```ts
startLocalFirstGraph({ storage, store, key: `aso:${session.practiceId}` });
```

The return value — which carries `dispose`, `persistNow`, `hydrate` — is **never captured**
(no `const handle =`, no assignment of any kind). The effect's teardown, `:96-99` verbatim:

```ts
    return () => {
      cancelled = true;
      void opened?.close();
    };
```

only sets `cancelled` and closes PGlite; `dispose()` is never called, and is in fact
unreachable because the handle was discarded. So even once PEM grows a proper barrier, ASO
never calls it: subscriptions leak and a disposed session's writes can still land. This is the
"late open resurrects a disposed session" hazard from §7 of the source architecture, present in
code today.

## G3 — Atomic rows + resume checkpoint: NOT MET (high)

`packages/entity-graph-core/src/adapters/pglite-persistence.ts` (102 lines) exposes only
`PGlitePersistenceClient`, `CreatePGlitePersistenceAdapterOptions` and
`createPGlitePersistenceAdapter`. There is **no transaction boundary and no checkpoint
surface** — nothing to make rows and resume position commit together.

The Electric adapter discards the resume position outright.
`adapters/electricsql.ts:90` constructs a change whose offset is the empty string:

```ts
          const change = toChange(tc as ElectricTableConfig<Record<string, unknown>>, {
            headers: { operation: parsed.op as "insert"|"update"|"delete" },
            offset: "", key: String(parsed.row[tc.idColumn ?? "id"]), value: parsed.row });
```

An empty offset cannot describe a commit boundary, so on resume there is nothing to reconcile
persisted rows against. ADR-009's requirement — "commit replica rows and resume checkpoint
atomically" — has no foundation to build on yet.

`must_refetch` handling is absent because **no consumer exists at all**: nothing in PEM or ASO
reads FRF's `GET /v1/shape`, so there is no module in which the signal could be handled. FRF
surfaces it (`ShapeChunk.must_refetch`, `crates/frf-shape-electric/src/facade.rs`) and the
gateway maps it to HTTP 409 plus `electric-must-refetch`. The gap is therefore not "a consumer
mishandles the signal" but "the consumer that must handle it is unwritten" — the rebuild-the-
generation and remove-stale-rows behaviour lands with the G4 materializer, not before it.

## G4 — Materializer + coherent publication: PARTIAL (high)

**A candidate atomic publication surface exists; its coherence is unverified.**
`packages/entity-graph-core/src/graph.ts:360`,
`ingestFetchedList`, is a single `set(...)` that commits, in one Zustand publication: the
primary entity batch, `options.sideBatches` ("rows that must commit or fail with the primary
batch"), `lists`, view-backed `projections`, and `finishListFetches`. ADR-009 hedged that this
might need building as a new core contract — **that surface is already present.** The seeded
goal was right to re-scope G4 toward *using* it. Whether it is sufficient remains open: it is
a single `set(...)`, which is necessary for atomic publication but not by itself proof of
subscriber coherence.

**Not yet proven:** ADR-009 requires coherence verified by *subscriber traces*, not only
rendered screens, because imperative subscribers observe Zustand directly and React batching
does not cover them. No such trace test exists.

**Missing entirely:** the materializer. ASO has `electric-shapes.ts` (230 lines) and
`pglite-schema.ts` (153 lines), but nothing consumes FRF's `GET /v1/shape` and lands rows into
real local tables. `graph-provider.tsx:76-78` creates a bare `PGlite()` and execs the schema
inline, with a comment marking it ephemeral — there is no worker DB owner, no migration ledger,
and no exclusive-ownership lease as §7 requires.

### G4a — `met` / `gap` / `void` are NOT distinguishable from "not loaded" (new; found in review)

The child's G4 requires: *"Keep `met`, `gap` and `void` distinct from 'not loaded.'"* The first
draft of this assessment did not evaluate that clause — the adversarial reviewer flagged the
omission, and investigating it surfaced a real defect.

The vocabulary exists and is load-bearing.
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

`EvidenceCounts` (`shared/model/evidence-state`) carries **exactly three** states. There is no
fourth "not loaded" variant, and a repo-wide search for `notLoaded` / `not_loaded` in
`web/src` returns nothing.

The consequence is specific: `countStates` seeds all three counters at zero, so a timeline
**mid-hydration** reports `{met: 0, gap: 0, void: 0}` — structurally identical to a
fully-loaded case that genuinely has no entries. The doc comment at `:110-112` says the zero
seeding exists so "a state with no entries reports `0` rather than being absent," which is
correct for a *loaded* case and precisely wrong for an unloaded one.

Downstream this feeds clinical text. `features/case-queue/model/case-summary.ts:20` documents
the intent: `{met: 4, gap: 3, void: 1}` should read "1 document to obtain." A partially
hydrated case therefore renders a confident clinical summary derived from rows that have not
arrived yet.

**Exit for G4a:** the state model gains an explicit not-loaded representation (or callers gate
on load state before counting), and a test proves a mid-hydration timeline is not reported as
a loaded all-zero case.

## Cross-cutting finding — storage key violates the source architecture

`web/src/app/providers/graph-provider.tsx:82`, verbatim:

```ts
        startLocalFirstGraph({ storage, store, key: `aso:${session.practiceId}` });
```

§7 of the source architecture (`docs/architecture/application-runtime-architecture.md` in the
`prior-auth` repo, commit `6bf36c2`) is explicit:

> The persisted namespace is based on deployment, practice, identity, authorization-scope
> revision and replica generation; the transient epoch prevents old work from crossing a
> session transition. Do not key private storage only by practice ID.

The key uses practice **alone**. Two users of the same practice on one device therefore share a
persisted namespace, and neither authorization-scope revision nor replica generation
participates — so a scope change or schema migration silently reuses stale private data.

**Status: confirmed against both the code and the source doc**, with both quoted above. It is
outside the child's four seeded goals and is proposed as an addition. It is a privacy boundary,
not a refactor, and it lives in the same file and concern as G1 (what identifies a runtime),
so folding it into G1 costs little.

## Open questions for plan

1. **Ordering.** G1 and G2 are prerequisites — a materializer built on unscoped runtime state
   and unreachable disposal cannot be made correct afterwards. Recommend G1 → G2 → G3 → G4,
   with the storage-key fix folded into G1 (same file, same concern: what identifies a runtime).
2. **Runtime handle plumbing.** Fixing G2 changes `startLocalFirstGraph`'s contract from
   fire-and-forget to a handle the caller must retain and await. Confirm whether other PEM
   consumers besides ASO call it, since that is a breaking change for them.
3. **Provisional schema.** ASO's privacy-approved replica schema (sequence step 1) still does
   not exist. G4's tables will be provisional; decide whether to build against the illustrative
   example catalog or defer G4's table shape until ASO defines it.
4. **`isSynced` semantics.** ADR-009 notes PEM's `isSynced` is based on pending actions, not
   Electric freshness. Once per-runtime state lands, decide whether relational sync readiness
   becomes a distinct signal rather than overloading `isSynced`.

## Verification constraint

The operator has directed that CI/CD is not used for testing and no testing occurs until all
code is written. G2's barrier, G3's crash consistency and G4's subscriber coherence are all
properties that require **execution** to prove. This phase can implement and type-check; it
cannot certify. Record what remains unproven rather than implying otherwise.


## Unresolved review findings

Adversarial review ran twice (`review/assess/findings.json`, `findings2.json`; judge
`kbd-judge`, `cross_model_check: verified-distinct`). Both rounds returned **BLOCK** on the
same CRITICAL, which is **accepted unresolved** at the protocol's 2-round cap. Recorded here so
plan sees it.

**CRITICAL (both rounds) — external-repo claims are not independently checkable from the packet.**
The judge is correct about what it received. `build-review-packet.sh` resolves `cited_paths`
against the repo it runs in, so all nine PEM/ASO citations reported `MISSING`, in both rounds.

Why it is accepted rather than fixed:

- **It is a tooling limitation, not an artifact defect.** No edit to this document can make the
  packet builder reach a sibling repository. Round 2 added verbatim source quotes for every
  load-bearing claim, and the CRITICAL repeated unchanged — confirming the finding is about
  packet assembly, not about what the assessment says.
- **The claims were verified**, by reading each file at the cited line
  (`sed -n '<line>p'`) before and after each review round. Every quote in this document is a
  literal copy of the source line.
- **A reader can now check them without the packet**, because the evidence travels inline.

Two follow-ups this implies, neither owned by this phase:

1. `build-review-packet.sh` should accept additional repo roots (e.g. `--repo-root`) so
   cross-repo phases can be reviewed on their evidence. Until then, every cross-repo KBD
   artifact will BLOCK for this reason regardless of quality.
2. Plan should treat these findings as *verified-by-the-author, unverified-by-the-judge* and
   re-confirm the four load-bearing line numbers before acting, since they will drift as the
   PEM and ASO files change.

Both WARNINGs from round 2 were **addressed**: G4's "exists and is correct" was overstated and
is now "candidate surface, coherence unverified"; the vague `must_refetch` claim now states
plainly that no consumer exists in either repo. Round 1's WARNINGs were also addressed — the
`met`/`gap`/`void` clause became finding G4a (a real defect the reviewer surfaced), and the
storage-key finding gained its §7 quote and an explicit confirmed status.


## Correction — 2026-09-06: the ASO replica schema exists

This document repeatedly states that ASO's privacy-approved replica schema "does not exist
(sequence step 1)". **That is wrong**, and the error propagated into the plan, ADR-009 and the
c006 change before it was caught.

It exists, in the `prior-auth` repo:

- `web/src/shared/sync/pglite-schema.ts` — five tables, `OMITTED_COLUMNS` recording every PHI
  exclusion as assertable data, ADR-007 behind it, and `pglite-schema.test.ts` failing on a
  sixth table or a reappearing omitted column.
- `web/src/shared/sync/electric-shapes.ts` — `SYNC_RELATIONS` (base tables, not views — measured
  against a live stack 2026-09-05) and `SYNC_COLUMNS`, which *is* the PHI boundary on the wire,
  verified against a canary row. `createTenantScopedElectricAdapter` fails closed.
- `practice_id` denormalized onto every synced row *because an Electric shape WHERE clause is
  flat and cannot join* — the schema was shaped for this facade's request model.

**How the error happened:** the ASO runtime architecture lists the schema as sequence step 1,
and I read "listed as step 1" as "not yet done" without checking the repo.

**What changes:** c006 is unblocked and re-scoped from "define clinical tables" to "conform
FRF's catalog to ASO's existing tables." G4's table shapes are no longer provisional. No ASO
schema change is needed, and none should be made to suit the facade — `OMITTED_COLUMNS` states
that removing an entry is "a decision about PHI, not a cleanup."

**What remains genuinely open:** whether ASO adopts the facade at all. It syncs Electric
directly today, and that path works and is measured. Recorded in
`docs/architecture/frf-shape-facade-integration.md` in the `prior-auth` repo.
