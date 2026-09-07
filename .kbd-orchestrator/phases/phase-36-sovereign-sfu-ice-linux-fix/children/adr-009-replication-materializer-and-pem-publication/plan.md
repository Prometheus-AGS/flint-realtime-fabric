# Plan — adr-009-replication-materializer-and-pem-publication

> Backend: OpenSpec. Analyze stage skipped (no `library-candidates.json`) — this is
> defect-repair plus one new component in existing codebases, not a build-vs-adopt decision.
> No evolver cycle.
>
> Line numbers re-confirmed at plan time per the assess handoff's warning; all six still hold
> (`local-first-runtime.ts:119/164/216`, `graph.ts:360`, `electricsql.ts:90`,
> `graph-provider.tsx:82`).

## What "done" means in this phase

The operator directive is that no testing runs until all code is written, and CI is not used for
testing. Two words are therefore used precisely below, and they are **not** synonyms:

| Term | Meaning | Achievable in this phase? |
|---|---|---|
| **DONE** | Code implemented, spec delta valid (`openspec validate`), and the named tests **authored and type-checking** in the tree | **Yes** |
| **PROVEN** | Those tests **executed and passing** | **No — deferred** |

Where a change says "a test proves X," that test must exist and compile for the change to be
DONE; its execution is what would make the behaviour PROVEN, and that is a later, explicit
certification step. A change is never DONE on the strength of an unwritten test, and is never
PROVEN merely because its tests were written.

## Two facts that shape the ordering

**1. G1 and G2 are breaking changes to a published package.** `pendingActions`,
`graphSyncStatusStore` and `startLocalFirstGraph` are all **public API**: re-exported from
`@prometheus-ags/entity-graph-core` (4.0.0) and again from `entity-graph-react`, whose
`useGraphSyncStatus` hook (`graph-store.ts:123`) reads the singleton directly. This answers the
assessment's open question 2 — the blast radius is not "ASO only." Changing these is a **major
version** move, and the react package must be updated in lockstep or its hook breaks.

**2. The dependency chain is strict, not stylistic.** Each change below is a prerequisite for
the next in a way that makes reordering incorrect rather than merely awkward:

```
c001 (per-runtime state + storage key)
   └─> c002 (dispose barrier)      needs a runtime instance to own the in-flight set
         └─> c003 (checkpoint)     needs a drainable writer to commit atomically with
               └─> c004 (materializer)  needs all three, and is the only consumer of the
                                        must_refetch / rebuild-generation behaviour
```

Building c004 first — the tempting order, since it is the visible deliverable — would land a
materializer publishing into a graph whose pending set a sibling runtime can wipe (c001), whose
writes cannot be drained on teardown (c002), and whose rows cannot commit with a resume point
(c003). Every one of those would have to be revisited.

## Changes

| # | Change | Repo | Gap | Depends on |
|---|---|---|---|---|
| c001 | `p37-c001-pem-runtime-scoped-state` | PEM | G1 + storage key | — |
| c002 | `p37-c002-pem-dispose-barrier` | PEM, ASO | G2 | c001 |
| c003 | `p37-c003-pem-checkpoint-atomicity` | PEM | G3 (commit atomicity) | c002 |
| c004 | `p37-c004-aso-replica-runtime` (schema-agnostic) | ASO, (FRF) | G4 + **G3 (`must_refetch` rebuild)** | c003 |
| c005 | `p37-c005-evidence-not-loaded-state` | ASO | G4a | — (parallel) |
| c006 | `p37-c006-aso-clinical-materialization` | ASO | G4 | c004 + **ASO schema (external)** |

`c005` is deliberately **independent**: the `met`/`gap`/`void` defect lives in the evidence
state model, not the replication path, so it can proceed in parallel and must not be sequenced
behind the materializer it will eventually protect.

### c001 — `p37-c001-pem-runtime-scoped-state`

**Repo:** PEM · **Gap:** G1 (critical) + the storage-key finding

Move `pendingActions` and sync status from module scope into the runtime instance created by
`startLocalFirstGraph`, and widen the persistence key beyond practice.

- Remove `const pendingActions` (`local-first-runtime.ts:119`); own it per runtime.
- The `pendingActions.clear()` at `:164` becomes an instance operation, so hydrating runtime B
  can no longer erase runtime A's un-settled writes.
- `isSynced` (`:208`, `:229`, `:263`) reads the owning runtime's set.
- `graphSyncStatusStore` gains per-runtime scoping, and `useGraphSyncStatus`
  (`entity-graph-react/src/graph-store.ts:123`) is updated in lockstep.
- Storage key composes deployment + practice + identity + authorization-scope revision +
  replica generation, per source-doc §7. ASO's `` `aso:${session.practiceId}` `` becomes a
  composed key.

**Done when:** two runtimes constructed in one process have independent pending sets, and a
test proves runtime B hydrating leaves runtime A's pending actions intact. A test proves two
identities in the same practice resolve to different storage keys.

**PROVEN requires execution:** nothing beyond running the authored unit tests — c001 has no behaviour that needs a browser or a real process.

### c002 — `p37-c002-pem-dispose-barrier`

**Repo:** PEM + ASO · **Gap:** G2 (critical) · **Depends on:** c001

Make disposal await in-flight persistence, and make it reachable from ASO.

- PEM: `:216`'s `void persistGraphToStorage(...)` retains its promise in a per-runtime
  in-flight set (which c001's instance state makes possible). `dispose()` becomes
  `async dispose(): Promise<void>`, awaiting or cancelling every tracked write.
- ASO: capture `startLocalFirstGraph`'s return value at `graph-provider.tsx:82` — today it is
  discarded, so `dispose()` is unreachable.
- **A React effect cleanup is synchronous and cannot await a promise**, so awaiting inside the
  teardown at `:96-99` would not actually provide a barrier. Introduce an explicit
  session/lifecycle manager that owns the runtime, records the disposal promise, and makes the
  *next* open await or supersede it. The effect teardown calls into that manager; it does not
  itself do the waiting.
- StrictMode: a double mount/unmount must not create two owners or let a late open resurrect a
  disposed session — which is precisely what the manager's supersede path exists to prevent.

**Done when:** a test proves no persistence write lands after the disposal promise resolves,
including a write started immediately before disposal; and ASO's teardown calls `dispose()`.

**PROVEN requires execution:** real StrictMode remount behaviour in a browser. The barrier's
unit tests are authored here; the mount/unmount race needs a running React tree.

### c003 — `p37-c003-pem-checkpoint-atomicity`

**Repo:** PEM · **Gap:** G3 · **Depends on:** c002

Give the persistence adapter a transaction boundary and a checkpoint, and stop discarding the
resume position.

- `pglite-persistence.ts` gains a transactional write committing rows **and** checkpoint
  together.
- `electricsql.ts:90` stops sending `offset: ""` and carries the real Electric offset, so a
  checkpoint can describe a commit boundary.
- Resume validates that persisted rows and checkpoint describe the same boundary; a mismatch
  rebuilds rather than merging.

**Done when:** rows and checkpoint commit in one transaction, and a test simulating an
interruption between the two leaves the replica resumable with no torn state.

**G3 is deliberately split.** c003 owns the *commit* half — rows and checkpoint land together,
and resume validates they describe the same boundary. The *rebuild* half —
`must_refetch` discarding a generation and removing stale rows — belongs to c004, because it
requires a consumer that fetches chunks, and no such consumer exists until then. **c003 is not
"G3 complete"**; it is G3's atomicity half, and G3 closes only when c004 lands.

**PROVEN requires execution:** real crash consistency. A simulated interruption is authored as a
unit test here; an actual process kill mid-transaction is not expressible as one, and is
deferred with all testing.

### c004 — `p37-c004-aso-replica-runtime` (schema-agnostic)

**Repo:** ASO (+ FRF only if the chunk contract needs changing) · **Gap:** G4 · **Depends on:** c003

Build the replica runtime that consumes `GET /v1/shape` and publishes through PEM's existing
atomic surface — **without hardcoding any clinical table**. Every table this change touches is
supplied by the caller from the shape catalog, so nothing here presumes a schema ASO has not
yet approved.

- A worker DB owner with an exclusive-ownership lease and a migration ledger, replacing the
  bare `new PGlite()` at `graph-provider.tsx:76-78` marked ephemeral in its own comment.
- A **generic** chunk→table writer driven by the shape's declared table and columns; it takes
  the target as data and has no knowledge of `prior_auth_request` or any other clinical table.
- Publish one database revision as **one** `ingestFetchedList` call (`graph.ts:360` — the
  surface already exists; do not build another).
- Handle `must_refetch`: rebuild the affected replica generation and **remove stale rows**
  rather than merging a new snapshot into old data. FRF already surfaces this
  (`ShapeChunk.must_refetch`, HTTP 409 + `electric-must-refetch`); this change is its first
  consumer.
- Referential consistency across a screen's required tables is enforced by the runtime's
  publication boundary, not by per-table knowledge.

**Done when:** chunks materialize into caller-declared tables and one revision publishes
atomically, exercised by a fixture schema in tests — **not** by any ASO clinical table.

**Also done when:** a **subscriber-trace test exists** asserting that a subscriber attached
across a publication observes either the previous complete projection or the next one, and
never a partially updated relationship. ADR-009 requires this be shown by traces rather than
rendered screens, because imperative subscribers observe Zustand directly and React batching
does not cover them. **The test being written is an acceptance criterion; the test passing is
certification** — c004 is not done without the trace test in the tree, even though it cannot be
executed under the current directive.

**Not blocked:** because this change is schema-agnostic, it proceeds without ASO's replica
schema. Concrete clinical tables are deferred to c006.

### c005 — `p37-c005-evidence-not-loaded-state`

**Repo:** ASO · **Gap:** G4a · **Depends on:** — (parallel with c001–c004)

Make "not loaded" distinguishable from a loaded case with no entries.

`countStates` (`evidence-timeline.tsx:113-119`) seeds `{met: 0, gap: 0, void: 0}`, so a
mid-hydration timeline is structurally identical to a fully-loaded empty case — and
`case-summary.ts:20` turns those counts into clinical text ("1 document to obtain"). Either add
an explicit not-loaded representation to `EvidenceCounts`, or gate callers on load state before
counting.

**Done when:** a test proves a mid-hydration timeline is not reported as a loaded all-zero
case, and no clinical summary text is derived from unloaded counts.

### c006 — `p37-c006-aso-clinical-materialization`

**Repo:** ASO · **Gap:** G4 (concrete tables) · **Depends on:** c004 **and an external input**

Bind c004's generic runtime to ASO's actual clinical tables: migrations, typed repositories and
the shape catalog entries that name them.

**BLOCKED — do not start.** ASO's privacy-approved replica schema is sequence step 1 and does
not exist. Writing migrations against a guessed schema would produce exactly the "provisional,
must be revisited" churn the assessment warned about, and would put unapproved clinical column
names into a migration ledger. `docs/examples/shape-catalog.example.json` is illustrative only
and is **not** approved schema.

**Unblocks when:** ASO publishes the privacy-approved replica schema and its shape catalog.

**Done when:** the declared clinical shapes materialize through c004's runtime, with migrations
recorded in the ledger and typed repositories over the resulting tables.

This change exists in the plan so the dependency is visible and tracked. It is expected to
remain unstarted for the duration of this phase.

## Ordering rationale

c001 → c002 → c003 → c004 is a **hard chain**: each change supplies state the next one needs
(instance ownership → drainable writer → atomic checkpoint → a consumer that relies on all
three). c005 is parallel because it touches a different subsystem. c006 sits after c004 but is
gated on an external input and is expected to stay unstarted.

**Deliverable count for this phase: 5** (c001–c005). c006 is tracked, not scheduled.

Apply c001 first. It is the only change with no prerequisite, it is where the storage-key
privacy fix belongs (same file, same question of what identifies a runtime), and every other
PEM change depends on the instance state it introduces.

## OpenSpec spec deltas (constraint P1 — required, not optional)

`.kbd-orchestrator/constraints.md:75-77` requires **every** change to carry a
`specs/<capability>/spec.md` delta (`## ADDED/MODIFIED/…` + a MUST/SHALL body line +
`#### Scenario:` blocks) so `openspec validate <id>` passes before archive. The constraints
table at `:91` applies P1 to Frontend/TS, which is all of c001–c006.

The first draft of this plan omitted this and was correctly BLOCKed for it. Each change below
therefore carries a delta as an **acceptance criterion**, not a follow-up:

| Change | Capability | Delta shape |
|---|---|---|
| c001 | `local-first-runtime` | MODIFIED — runtime-scoped pending actions and status; composed storage key |
| c002 | `local-first-runtime` | MODIFIED — `dispose()` becomes async and drains in-flight persistence |
| c003 | `replica-persistence` | ADDED — rows and resume checkpoint commit atomically |
| c004 | `replica-materialization` | ADDED — schema-agnostic chunk→table runtime with atomic publication |
| c005 | `evidence-state` | MODIFIED — not-loaded is distinguishable from a loaded empty case |
| c006 | `replica-materialization` | MODIFIED — concrete clinical tables (deferred with the change) |

**Gate for every change:** `openspec validate <id>` passes before archive. A change whose only
remaining work is its spec delta is **not** done.

Note the FRF-side precedent and its limit: `p36-c003` used `skip_specs: true` because it was
pure tooling and docs. None of c001–c006 qualifies — they all change behaviour a spec should
describe, so none may use that escape.

## Semver and coordination

c001 and c002 change public API on `@prometheus-ags/entity-graph-core` (4.0.0) and
`entity-graph-react`. Both packages move together, and the version bump is **major**. The
existing `local-first-runtime.test.ts` is the home for c001–c003 regression tests.

## Verification posture

Per the operator directive, no testing runs until all code is written; CI is not used for
testing. Every change above therefore lands **DONE but not PROVEN**, in the precise sense
defined at the top of this plan: implemented, spec-delta-valid, with its tests authored and
type-checking — but with those tests unexecuted.

Each change's "PROVEN requires execution" note names exactly what its authored tests would
establish once run, so certification is a deliberate later step against a written list rather
than a vague future intention. **No change in this phase may be reported as verified,
certified, or proven.**


## Adversarial review record

Two rounds (`review/plan/findings.json`, `findings2.json`; judge `kbd-judge`,
`cross_model_check: verified-distinct`). **All findings from both rounds were addressed** — no
finding is carried unresolved.

**Round 1 — 3 CRITICAL, 1 WARNING:**

1. *P1 spec deltas omitted despite the OpenSpec backend.* Correct and material: the first draft
   simply left them out, and `constraints.md:75-77` applies P1 to Frontend/TS. Added a required
   spec-delta table with `openspec validate <id>` as a per-change gate, and noted why the
   `skip_specs` escape used by `p36-c003` does not apply to any change here.
2. *c004 could be marked done without G4's subscriber-coherence proof.* Correct: coherence was
   filed under "cannot be proved" rather than required. The subscriber-trace test is now an
   acceptance criterion — authored for DONE, executed for PROVEN.
3. *c004 planned schema-specific materialization while the schema is explicitly absent.*
   The sharpest finding of the review. Split into c004 (schema-agnostic runtime, proceeds now,
   tested against a fixture schema) and c006 (concrete clinical tables, blocked on ASO). This
   removed a real contradiction: the plan had acknowledged the schema does not exist while
   scheduling work that needs it.
4. *WARNING — React effect cleanup cannot await a promise.* Correct on the language: cleanup is
   synchronous, so "await dispose() in the teardown" would not have produced a barrier. c002 now
   specifies an explicit session/lifecycle manager that owns the disposal promise and makes the
   next open await or supersede it.

**Round 2 — 1 CRITICAL, 1 WARNING:**

5. *Acceptance criteria demanded proof while the plan said nothing would be run.* A genuine
   internal contradiction. Resolved by defining **DONE** (implemented, spec-valid, tests
   authored and type-checking) and **PROVEN** (tests executed) as distinct terms up front, and
   using them consistently.
6. *WARNING — G3's `must_refetch` behaviour was moved into c004 without reflecting the split.*
   Correct bookkeeping error: the gap table implied c003 closed G3. It now shows c003 as G3's
   atomicity half and c004 as carrying G3's rebuild half, and c003 states explicitly that it is
   not "G3 complete."


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
