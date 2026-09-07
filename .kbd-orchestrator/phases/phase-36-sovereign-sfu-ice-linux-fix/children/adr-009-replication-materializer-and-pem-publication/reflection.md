# Reflection — adr-009-replication-materializer-and-pem-publication

> Child of `phase-36-sovereign-sfu-ice-linux-fix`. Reflected 2026-09-07.
> **Implementation 5/5 COMPLETE. Evidence BLOCKED. Certification PENDING.**
>
> Those three lines are not the same claim, and the gap between them is the most
> important thing in this document: every line of code landed, and **not one test
> has been executed**.

## Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| G1 — per-runtime state | **MET (unproven)** | `RuntimeScope` owns pending actions + status; PEM `97baec05` |
| G2 — async dispose barrier | **MET (unproven)** | in-flight set + async `dispose()`; PEM `4614976b`, ASO `eb03edd` |
| G3 — atomic rows + checkpoint | **MET (unproven)** | commit half PEM `297cccea`, rebuild half ASO `e1622d9` |
| G4 — materializer + coherent publication | **PARTIAL** | runtime built schema-agnostic (ASO `e1622d9`); catalog binding is c006, unstarted |
| G4a — `met`/`gap`/`void` vs not-loaded | **MET (unproven)** | `EvidenceTally`; ASO `2a0bb5a` |
| Storage key (proposed addition) | **MET (unproven)** | composed key; ASO `76556bb` |

**"MET (unproven)" is deliberate wording.** The code exists and type-checks; the
behaviour is unverified. A goal whose test has never run is not demonstrated,
and calling it MET without the qualifier would be the exact overclaim this
phase's own review process caught twice.

**G4 is PARTIAL, not MET.** The replica runtime exists and is schema-agnostic,
but nothing yet binds it to ASO's real tables — that is c006, which this phase
unblocked but did not execute.

## Delivered

| Change | Repos | Spec delta | Archived |
|---|---|---|---|
| c001 runtime-scoped state + storage key | PEM, ASO | valid | **no** |
| c002 dispose barrier + session manager | PEM, ASO | valid | **no** |
| c003 checkpoint atomicity | PEM | valid | **no** |
| c004 schema-agnostic replica runtime | ASO | valid | **no** |
| c005 evidence not-loaded state | ASO | valid | **no** |
| c006 catalog binding | — | — | unstarted (unblocked) |

Seven commits across three repos: FRF ×4 ledger/doc, PEM ×3, ASO ×4.
All five spec deltas pass `openspec validate` (constraint P1).

**Nothing is archived, on purpose.** Archiving reads as certified, and 85
authored tests have never executed. Leaving them in `openspec/changes/` keeps the
distinction visible instead of burying it.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/5** |
| First-pass pass rate | n/a — gate never ran |
| Changes requiring refinement | unknown |
| Adversarial reviews run | 2 (assess, plan) |
| Adversarial findings addressed | 8 of 9 (1 accepted unresolved) |

**The artifact-refiner QA gate did not run on any change in this phase.**
`.refiner/artifacts/` contains logs for `p17-*` and nothing for `p37-*`. The
`/kbd-apply` loop documents a QA gate after the last task; I ran `verify` and
`archive` but never the refiner. That is a process gap I introduced, recorded
here rather than left for someone to discover from an empty directory.

### Recurring constraint violations

None recorded — the gate that would record them never ran. This is an absence of
data, **not** a clean result, and should not be read as one.

## What adversarial review actually caught

Two rounds each on assess and plan. It was worth the cost, and the findings were
not cosmetic:

1. **G4's "not loaded" clause had gone unassessed** (assess, WARNING). Chasing
   the omission found a real defect: `blockedOn` returns **"Ready to draft"** for
   a mid-hydration case, a clinical claim derived from rows that had not arrived.
   That became c005 — an entire change that would not otherwise exist.
2. **The plan scheduled a materializer against a schema it said did not exist**
   (plan, CRITICAL). Splitting c004 (schema-agnostic, ships now) from c006
   (catalog binding) removed a contradiction the plan had been carrying.
3. **"Await `dispose()` in the effect teardown" would not have worked** (plan,
   WARNING). React cleanup is synchronous. Implementing c002 confirmed the catch;
   the barrier had to move to a session manager the *next* open consults.
4. **P1 spec deltas were omitted entirely** (plan, CRITICAL) despite the phase
   using OpenSpec.
5. **Acceptance criteria demanded proof while the plan said nothing would run**
   (plan, CRITICAL) — resolved by defining DONE vs PROVEN as distinct terms, the
   vocabulary this reflection depends on.

One finding was **accepted unresolved**: adversarial review BLOCKs every
cross-repo artifact, because `build-review-packet.sh` resolves `cited_paths`
against the running repo only. Round 2 added verbatim source quotes and the
finding repeated unchanged, confirming it is a tooling limit rather than an
artifact defect.

## Corrections I made to my own work

Recorded because a phase that only lists successes teaches nothing:

- **I claimed ASO's replica schema did not exist.** It did — five tables, PHI
  exclusions as assertable data, ADR-007, and a test that fails on a sixth table.
  I inferred absence from the runtime architecture's sequence table without
  checking the repo, and the error propagated into the assessment, plan, ADR-009
  and c006 before being caught. Corrected in all four.
- **I misread the `offset: ""` defect.** The plan said "stop sending it"; the
  real defect was that `toChange` read `msg.offset` and **discarded it**, so no
  offset reached a consumer by any path. The `""` I flagged turned out to be
  correct — a LISTEN/NOTIFY frame genuinely has no Electric offset.
- **I reported a typecheck as passing that never ran.** `tsc` is not resolvable
  from the ASO repo root; my "OK" was grep's exit code. Same false-green class as
  the rustup incident earlier in this session.

## Technical debt introduced

| Debt | Where | Why accepted |
|---|---|---|
| 85 tests never executed | all five changes | Operator no-testing directive |
| No artifact-refiner QA | all five changes | Process gap — I did not run the gate |
| Nothing archived | all five changes | Deliberate; archiving would read as certified |
| `graphSyncStatusStore` is last-writer-wins under multiple runtimes | PEM | Legacy export kept working; per-runtime status is the new path |
| Migration DDL + ledger insert are two statements | PEM ledger | PGlite cannot mix DDL and parameterised DML in one call; migrations must stay idempotent |
| Authorization-scope revision absent from the storage key | ASO | `VerifiedSession` has no revision counter; hashing capabilities would churn the namespace |
| Major version not bumped | PEM | Release action, and this code is unproven |

## Lessons

1. **"Listed as a future step" is not evidence it has not been done.** The
   schema error cost four artifacts' worth of corrections. Check the repo.
2. **A task description can be wrong about its own defect.** T4 named a symptom;
   the cause was a discarded value three functions away. Re-read the code the
   task points at before implementing what the task says.
3. **Adversarial review pays for itself on artifacts, not just diffs.** The two
   most valuable findings — c005 existing at all, and the c004/c006 split — came
   from reviewing an assessment and a plan, before any code was written.
4. **A framework constraint can invalidate a plan step.** React's synchronous
   cleanup meant the planned barrier could not exist where the plan put it. Worth
   checking framework semantics at plan time, not implementation time.
5. **Exit codes through pipes lie.** Twice this session a passing status came
   from a grep rather than the tool. Capture the tool's own exit code.

## Recommended focus for the next phase

**Not more implementation. Verification.**

Five changes, 85 tests, zero executions. The single highest-value action is to
run them — particularly:

- c004's **subscriber-trace test**, which ADR-009 specifically requires be *run*
  before publication coherence may be claimed.
- c003's interruption test and c002's barrier test, both of which assert crash
  and race behaviour that only execution demonstrates.

After that, in order:

1. **Run the artifact-refiner QA gate** on all five changes, then archive them.
2. **Decide whether ASO adopts the facade at all** — it syncs Electric directly
   today and that path works and is measured. c006 ships or is withdrawn on that
   answer; it should not be implemented before it.
3. **Fix `build-review-packet.sh`** to accept extra repo roots, or every future
   cross-repo artifact BLOCKs for a reason unrelated to its quality.

## Parent phase

`p36-c002` (CI decode run + `SFU_MODE` flip) remains open and untouched by this
child — the child's `scope.json` explicitly denied the media path. Returning to
it needs `/kbd-next-child`. Note that `p36-c002` is itself blocked on the same
no-testing directive, since it is a pure run-and-observe change.
