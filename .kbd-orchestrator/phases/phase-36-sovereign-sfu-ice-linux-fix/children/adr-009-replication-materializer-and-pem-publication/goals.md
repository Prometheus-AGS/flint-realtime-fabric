# Goals — phase-36-sovereign-sfu-ice-linux-fix› adr-009-replication-materializer-and-pem-publication

> Seeded 2026-09-06 from the ADR-009 shape-facade work (FRF commit `83d7c23`). The FRF read
> path — port, adapter, Keto authorization, `GET /v1/shape` — is implemented and unit-tested.
> What remains is the **replica side**: the materializer that lands rows locally and the PEM
> publication contract that makes them visible coherently.
>
> **This is cross-repo TypeScript work**, not Rust. Source of truth is
> `docs/architecture/application-runtime-architecture.md` in the `prior-auth` repo
> (commit `6bf36c2`), §7–§8, and ADR-009's "Relational replication" section.

## Repos in scope

| Repo | Path | Why |
|---|---|---|
| PEM | `../../prometheus-entity-management` | Owns the graph, runtime and persistence adapters (sequence step 3) |
| ASO web | `../../../TribeHealth/kevin/prior-auth/web` | Owns the worker DB, migration ledger and materializer (steps 4–5) |
| FRF | this repo | Only the shape-facade surface, if a real consumer needs chunk/checkpoint changes |

**Out of scope, explicitly denied:** the sovereign-SFU media path and `decode-proof.yml`. The
parent phase's `p36-c002` gate (CI decode run + `SFU_MODE` flip) is still open and must not be
disturbed by replication work.

## Findings that shape this phase (verified 2026-09-06)

**PEM already has the atomic publication surface.** ADR-009 hedged — "if PEM lacks this batch
surface, add it as a core contract before wiring replication" — but it does not lack it.
`ingestFetchedList` (`packages/entity-graph-core/src/graph.ts:360`) is a **single `set(...)`**
committing entities, lists, view-backed projections and side-batches in one Zustand
publication, with `IngestFetchedListOptions.sideBatches` documented as "rows that must commit
or fail with the primary batch." **G4 below is therefore about *using* this surface correctly,
not building it.** Verify subscriber-trace behaviour before assuming it is sufficient — ADR-009
requires proof by subscriber traces, not only rendered screens.

## G1 — Scope PEM runtime state per runtime

**Problem:** `packages/entity-graph-core/src/local-first-runtime.ts:119` holds
`const pendingActions = new Map<string, GraphActionRecord>()` at **module scope**. Two runtimes
in one process share it, so account or practice replacement cannot be supported without a full
page reload.

**Exit:** pending actions and sync status are owned per runtime instance. Two concurrently
constructed runtimes observe independent pending sets. A test proves one runtime's pending
actions are invisible to the other.

## G2 — Async dispose barrier

**Problem:** `dispose()` (`local-first-runtime.ts:270`) is **synchronous**: it unsubscribes and
clears a timer, but provides no barrier for in-flight persistence. A disposed session's write
can still land after teardown — the "late open resurrects a disposed session" hazard named in
§7 of the source doc.

**Exit:** disposal returns a promise that resolves only once every in-flight persistence
operation has settled or been cancelled. A test proves no write lands after the barrier
resolves, including a write started immediately before disposal.

## G3 — Atomic rows + resume checkpoint

**Problem:** `adapters/pglite-persistence.ts` has essentially no checkpoint handling. ADR-009
requires replica rows and the resume checkpoint to **commit atomically**, so that on resume the
persisted rows and the checkpoint describe the same commit boundary.

**Exit:** rows and checkpoint commit in one transaction. A crash-consistency test — interrupt
between row write and checkpoint write — leaves the replica resumable with no torn state.
Electric's `must_refetch` rebuilds the affected replica generation and **removes stale rows**
rather than merging a new snapshot into old data.

## G4 — Materializer + coherent PEM publication

**Exit:** shape chunks from `GET /v1/shape` materialize into real local tables, and one database
revision publishes as **one** atomic PEM store update. Subscribers observe either the previous
complete projection or the next one — never a partially updated relationship. Verified by
subscriber traces, not only by rendered screens (React batching alone is insufficient, because
imperative subscribers also observe Zustand).

Initial multi-table hydration must preserve referential consistency across a screen's required
tables: do not expose half a citation/document relationship as a final `void`. Keep `met`, `gap`
and `void` distinct from "not loaded."

## Verification constraint

The operator has directed that **CI/CD workflows are not used for testing, and no testing occurs
until all code is written**. G3's crash consistency and G4's subscriber coherence are properties
that require execution to prove, so this phase can implement and type-check but **cannot certify**.
Record what remains unproven rather than implying otherwise.

## Blocked dependency

ASO's **privacy-approved replica schema** (sequence step 1) does not exist yet. Any schema this
phase materializes is provisional and must be revisited when ASO defines it. Do not treat the
example catalog in `docs/examples/shape-catalog.example.json` as approved schema — its table and
column names are illustrative only.
