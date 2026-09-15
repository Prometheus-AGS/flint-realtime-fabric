# Reflection — issue-triage-and-architecture-audit

_Written 2026-09-14. Six changes planned, six implemented, archived, committed and pushed.
The phase ledger read 5/6 when this was written; it was repaired on 2026-09-15 and now
reads 6/6 — see "The ledger was wrong; the schema was the cause"._

## Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| G1 — unblock local integration testing (#6) | **MET** | c001 (`3437b95`). `cargo test -p frf-broker-iggy -- --ignored` completes against a live server; server logged `Accepted new TCP connection` + `Created new session`. Issue #6 closed. |
| G2 — prove or disprove the #2 fix | **MET** | c002 (`851b328`). Guard observed passing, then observed **failing** under sabotage with the verbatim issue-#2 error. Issue #2 closed. |
| G3 — fix #7 (non-UUID tenantId) | **PARTIAL** | c003 (`3af3e93`) changed the literal to a UUID, but the exit criterion is "publishes successfully" and no publish was ever observed. Issue #7 deliberately left **open**. |
| G4 — reconcile CLAUDE.md against the ADRs | **MET** | c005 (`5f7a37e`). All four Open Decisions rows verified against ADR *and* code, then rewritten as Resolved. |
| G5 — hunt vacuous guards and dead code | **MET** | c006 (`0529fc9`). `ws.rs` deleted (orphan proven by build); two more vacuous guards found and labelled; two public fns deliberately kept. |
| G6 — repair the file-size violation | **MET** | c004 (`139d3c7`). `main.rs` 504 → 425; guard added, sabotage-tested, and its own blind spot found and fixed. |

Five MET, one PARTIAL. G3 is not rounded up: static proof is not the exit criterion that
was written, and the issue stays open to say so.

## The finding that matters

**Five vacuous guards now, not three.** The phase charter recorded three; c006 found two
more, and git history settles that neither rotted — they were *born* empty:

| Test | Assertions, every revision | Since |
|---|---|---|
| `frf-gateway/tests/subscribe_mux.rs` | 0 across 3 revisions | `bfa764b` |
| `frf-postgres-cdc/tests/cdc_integration.rs` | 0 across 2 revisions | `abc6587` |

`subscribe_mux.rs` is a comment outline ending in `println!("…skipped")` — it would pass
against a *fully live* stack. `cdc_integration.rs` has real wiring but prints
`published.len()` instead of asserting on it, and never performs the INSERT it sleeps
waiting for, so 0 is the ordinary outcome — and still a pass. Both `#[ignore]` strings
blamed missing infrastructure; that was misleading, and they now name the real defect.

The pattern across all five: **a test whose failure mode was never observed.** The
codebase's own standard — "a guard that has never failed is a hypothesis" — is the right
one, and it caught every instance once applied deliberately.

## Corrections I made to my own work

Recorded because the corrections, not the successes, are what this phase is for.

1. **G1's charter hypothesis was wrong, and I nearly inherited it.** The goals file
   asserts a client/server version-mismatch story (0.6.203 vs 0.4.214 vs 0.8.13). The two
   "different servers" were the same image `sha256:f7c54e3042254`; 0.8.13 was the CLI
   binary. Actual cause: **IPv6-first name resolution** — `localhost` resolved to `::1`
   while the server bound v4. A repeat of phase-36's costliest lesson.
2. **The charter's claim that "sibling TS/Go/C# clients already use a valid UUID" is
   false.** None of them sends a `tenantId` at all. Corrected in the proposal, plan,
   assessment and on issue #7.
3. **I called `openspec archive --json` a "non-destructive probe."** It archived
   `p0-c001`. Reverted via git; `git status openspec/` returned to clean.
4. **I claimed the gateway "returns HTTP 000 / unreachable."** It returns 401. Corrected
   across five artifacts.
5. **My own T3 sabotage in c002 was inert** — `frf-broker-iggy` has no dependency on
   `frf-postgres-cdc`, so the file I was told to corrupt was unreachable from that test
   binary. Ran it anyway to prove inertness, then wrote one that bit. *A sabotage that
   cannot fail the guard is as empty as a guard that cannot fail.*
6. **My own file-size guard had a blind spot** — `git ls-files` sees tracked files only,
   so a brand-new oversized file passed until staged. Found by sabotage, fixed, re-proved
   with an unstaged probe.
7. **A confounded probe nearly became a false finding** in c004 — a `.gitignore` test ran
   while a previous probe was still on disk. Re-run from a verified baseline: PASS.
8. **"Docker is UP" in my own position reminder was wrong.** Every running container
   belongs to other projects, including ones matching `flint`
   (`aso-prior-auth-flint-gate-*`). I nearly credited an open 5432 as this project's CDC
   fixture; it belongs to none of the three Postgres containers, which publish on
   5433/55432/34322.
9. **I reported that `prometheus kbd` lacks `migrate`/`change`/`task`/`stage`.** It has
   all four. I read a truncated `--help` and concluded too much from it.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 6/6 |
| First-pass pass rate | 6/6 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |
| Constraint violations | 0 |

### Recurring constraint violations

None — zero failed constraints phase-wide.

**This number is weaker than it looks.** "ALL PASS" reflects the *local* constraint gate
only. **Adversarial review did not run for a single artifact in this phase** — the LiteLLM
proxy returns HTTP 401 (reachable, no Authorization header configured); preflight reports
`providers_detected: []` and `distinct_models: 2` across three roles. All six changes are
`pending_review`. No cross-model judge has examined any artifact. A 100% first-pass rate
with no independent reviewer is a self-assessment, not a certification.

## The ledger was wrong; the schema was the cause

> **Updated 2026-09-15.** This section originally ended "the ledger stays at 5/6." It no
> longer does: the operator directed a schema repair, and the counter now reads **6/6** and
> validates. The original refusal is preserved below because it was correct on the evidence
> then available — the fix was a *schema conversion*, which is a different and larger action
> than the counter bump I was asked for on 2026-09-14, and forcing the number without it
> would have written a value the structure could not support.

As written on 2026-09-14, `progress.json` read `completed: 5, total: 6`; `current-waypoint.json` read
`implementationCompleted: 0` with `exactNextCommand: /kbd-apply p38-c001`. Both contradicted
git, which showed six commits on `origin/main`, zero active openspec changes, and six
archived `tasks.md` with **zero open boxes**. (`progress.json` has since been repaired; the
waypoint has not — see the end of this section.)

The root cause was not a stale number. `progress.json` declared `schemaVersion: "2"`, whose
invariant is **array-of-objects only** — every `.changes[]` must be an object with `.id`,
`status` and `implementation_status`. This ledger's `.changes` was an array of **strings**.
`kbd_progress_validate` therefore failed on the *untouched* file, before any edit. Verified
directly, not inferred.

Three paths, all rejected:

| Path | Why not |
|---|---|
| `kbd_progress_mark_implementation_complete` | Refused with `completion invariant failed`, exit 1, wrote nothing. Correct behaviour. |
| `prometheus kbd migrate --apply` | Inventory proves it **would not fix the counter** (`resultingCompleted: 5`) while rewriting 39 progress files, with 11 uncertain rows and 2 alias conflicts. |
| `prometheus kbd change register/transition` | Would work, but costs **twelve typed mutations** with `--title` and `--sequence` values I would be inventing now — reconstructing an event history that was never recorded, into a durable store outside the repo that git cannot revert. It also flips the project from `mode: legacy` into runtime authority for all 39 phases. |

Hand-editing `progress.json` was never an option: the skills forbid it, and the invariant
exists precisely to stop a number being forced past the structure that should derive it.

**On 2026-09-14 the ledger stayed at 5/6 and this reflection said so.** `progress.json` was
left byte-identical to its pre-run state (verified by diff).

### Resolution, 2026-09-15

The operator directed the schema repair. `.changes` was converted from six strings to six
v2 object rows carrying real per-change truth (`status: DONE`, `implementation_status:
COMPLETE`, `tasks_done`/`tasks_total` of 6/5/3/6/5/6, all verified against the archive),
with the mirror counters moved to 6/6 in the **same atomic pass** — the invariant requires
`count(implementation_status == COMPLETE) == impl_done`, so rows and counters cannot move
separately. The candidate was validated *before* it replaced anything; the live file
validates; `kbd_progress_mark_implementation_complete` now exits 0.

The shape was copied from a ledger that actually passes validation
(`graph-explorer/.kbd-orchestrator/phases/graph-explorer-ui/progress.json`), not invented.
No top-level key was added or dropped and change-id order is preserved — verified by diff.

**The helper was then sabotage-tested rather than trusted.** Its first exit-0 proved only
idempotency, since the file was already correct. Setting c006 back to `PENDING` with the
counter at 5 and re-running it moved 5 → 6 and validated: it genuinely drives the
transition.

**New defect found by that sabotage:** the helper sets `implementation_status` but **never
reconciles `.status`**, so the restored row read `IN_PROGRESS/COMPLETE`. The v2 invariant
does not catch this — a row may carry `PENDING`/`COMPLETE` and still validate. The live
file is consistent only because the conversion wrote `DONE` explicitly.

**Still not done, deliberately:** `current-waypoint.json` remains at
`implementationCompleted: 0` with `exactNextCommand: /kbd-apply p38-c001`. `waypoint.sh`
only *reads* that field to render a display string (`:61`); the sole writers are
`kbd-new-phase.sh` and `kbd-next-phase.sh`, i.e. advancing to a next phase. There is no
sanctioned in-place refresh, and hand-editing the waypoint would repeat the exact error
this section is about.

## Technical debt

1. ~~**`progress.json` is schema-non-conformant**~~ — **RESOLVED 2026-09-15.** Converted to
   six v2 object rows; counter reads 6/6 and validates. See "Resolution, 2026-09-15".
1a. **`kbd_progress_mark_implementation_complete` never reconciles `.status`** — it sets
   `implementation_status` only, so a row can end up `IN_PROGRESS`/`COMPLETE`. Found by
   sabotage-testing the helper, not by reading it. The v2 invariant does **not** catch the
   contradiction: a `PENDING`/`COMPLETE` row validates cleanly. Any future caller must set
   `.status` itself, as the 2026-09-15 conversion did.
1b. **`current-waypoint.json` has no sanctioned in-place refresh** — it still reads
   `implementationCompleted: 0` and points at c001 while `progress.json` reads 6/6.
   `waypoint.sh:61` only *reads* the field; the only writers are `kbd-new-phase.sh` and
   `kbd-next-phase.sh`. A projection that can go stale with no way to re-derive it is the
   root of this phase's nine position-reminder staleness events.
1c. **`reflect_complete` vs `reflection_complete`** — the orchestrator disagrees with
   itself. Live code reads `reflect_complete` (`rollup.sh:58`, `kbd-child-exit.sh:100`,
   both phase-creation scripts); `progress.schema.json:47` and
   `kbd-next-phase.sh:270` write `reflection_complete`. Left alone deliberately: renaming
   would break the readers. `additionalProperties: true` means both validate.
2. **`ci.yml:50-61` runs `cargo test --all`** — violates the non-negotiable "CI/CD is never
   used to run tests. Ever." Predates the policy (`6e549e8`), so drift, not defiance.
3. **`ci.yml:45` is red today** — `clippy --features dev-endpoints` fails `E0063` at
   `routes/dev.rs:161`. Proven pre-existing by stashing against unmodified HEAD.
   Default-feature clippy is green.
4. **`dagger/codegen.ts` stage 4 is doubly unsatisfiable** — runs FRB against a UniFFI
   crate whose `#[uniffi::export]` makes FRB's parser panic, and diffs against a gitignored
   file that can never exist. Flagged in c005, not fixed.
5. **The local stack cannot start** — compose interpolates the whole file and
   `FLINT_GATE_JWT_SECRET` has no value (no `.env`, only `.env.example`). Supplying it was
   declined; it is a secret. This blocked re-running the two proven Iggy tests in c006.
6. **Two tests need real assertions** — `subscribe_mux.rs` and `cdc_integration.rs`.
7. **Issue #8 remains open** — 46 archived changes with unticked tasks; the
   `openspec validate --archived` lint is wired to nothing.

## Lessons

1. **A phase charter is an artifact like any other, and it rots.** G1's version-mismatch
   hypothesis and the "siblings use a valid UUID" claim were both wrong *in the goals
   file*. Assessment re-derived them from code instead of inheriting them — that is why
   c001 found IPv6 rather than chasing versions for a second session.
2. **Sabotage the sabotage.** c002's prescribed sabotage was inert against a crate that
   does not depend on the file it corrupted. Verifying that the *test of the guard* can
   fail is a distinct step from verifying the guard.
3. **Count the defining file.** Excluding it produced a false `fetch_and_cache` finding;
   including it surfaced three real candidates, of which only one was dead. The corrected
   sweep is only safe when paired with judgment — two of the three were public API.
4. **A refusal is information.** The progress helper, the openspec validator and the
   file-size guard each refused something this phase. Every refusal was correct, and each
   one pointed at a real defect rather than an obstacle to route around.
5. **Do not fabricate state to satisfy a counter.** The ledger fix and the
   `FLINT_GATE_JWT_SECRET` question were the same question twice: would I manufacture data
   to make a number or a test look right? Both times the answer had to be no, and both
   times the honest record is more useful than the green one.

## Recommended focus for the next phase

**`kbd-ledger-schema-and-runtime-authority`**

1. Decide, with the operator, whether this project adopts runtime authority or stays
   legacy. Everything else depends on it, and it is not a decision an agent should make as
   a side effect of a counter update.
2. Repair `progress.json` to match its declared `schemaVersion: "2"` — convert string rows
   to object rows with `id`/`status`/`implementation_status`, then let the sanctioned
   helper derive counters and validate.
3. Fix the waypoint staleness class. This phase's position reminder went stale at **nine**
   transitions; nothing in `end-task` or `archive` rewrites it.
4. Configure the adversarial-review proxy, or record explicitly that this project ships
   without cross-model review. Six consecutive `pending_review` changes is a standing gap.
5. Then: `ci.yml`'s `cargo test --all` (policy violation) and the `E0063` red job.

Deferred deliberately: issue #7 (needs a live publish, which needs the stack, which needs
the secret) and issue #8 (its own change).
