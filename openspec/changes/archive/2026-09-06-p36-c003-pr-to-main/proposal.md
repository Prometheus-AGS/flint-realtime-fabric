# p36-c003 — Retire the proof-branch workflow; land the gate decision on main

## Summary

The original c003 proposed opening a PR from `sovereign-sfu-decode-proof` → `main`. That plan is
obsolete: **all sovereign SFU work is already merged into `main`**, and the proof branch no longer
exists. This change retires the branch-based proof workflow and records the phase-36 gate decision
directly on `main`.

## Why the original plan no longer applies

The original proposal rested on three premises, all now false:

| Original premise | Actual state |
|---|---|
| `main` has been untouched since phase-0 | `main` carries all phases 0–36; merged via PRs #3 and #4 |
| The proof branch holds all infrastructure | The branch was deleted after merge; nothing was lost |
| A PR is needed to land the work | Already landed — `9ba04ae` (the last decode-proof run's head) is an ancestor of `main` |

Verification: `git merge-base --is-ancestor 9ba04ae origin/main` succeeds, and the ICE-lite fix from
the final proof-branch commit is present at `crates/frf-media-str0m/src/session.rs:229`.

There is no code to merge. A PR would be empty.

## The real remaining gap: a dead CI trigger

`.github/workflows/decode-proof.yml:21-22` scopes its `push` trigger to `branches:
["sovereign-sfu-decode-proof"]` — a branch that no longer exists. That trigger is now dead, so
pushes can never run the decode proof.

`workflow_dispatch` still works: the workflow is present on `main` (the default branch), which is
where GitHub registers dispatch triggers. **This is how c002 must now trigger its CI run** — the
mechanism c002's summary attributes to "push to `sovereign-sfu-decode-proof` handled by c001's T5"
no longer functions.

The workflow's own comment anticipated this: "Remove/limit before merge so it does not run the heavy
stack on every push to main." The merge happened; the cleanup did not.

## Scope

This change does **not** flip `SFU_MODE`, and does not depend on the gate outcome — it is
housekeeping that is correct whether c002 flips or holds. Retiring a dead trigger and recording an
honest phase record are both valid with `framesDecoded=0`.

## Gate relationship (changed from original)

The original was **blocked on c002's flip**. This rewrite is **not gate-blocked**, but is ordered
after c002 so the phase record it writes reflects c002's actual decision. If c002 holds the gate
OFF, this change still completes — it records the hold.

## Current evidence (run 29112243615, 2026-07-10)

The most recent decode-proof run advanced the failure downstream:

- `ice=connected` — **ICE now completes on Linux**; the ICE-lite fix resolved the phase-36 stall
- `localCandidates=1 remoteCandidates=1`
- `framesDecoded=0` — the gate correctly held; step 10 "Run the decoded-media proof" failed

Phase-36 goals G2 (diagnose ICE stall) and G3 (fix candidate pairing) are therefore **met**. The
open problem is no longer ICE — it is the media path after ICE connects, which is c002's territory.

## Files

| File | Change |
|---|---|
| `.github/workflows/decode-proof.yml` | Remove the dead `push:` trigger scoped to `sovereign-sfu-decode-proof`; keep `workflow_dispatch` as the sole trigger |
| `docs/PHASE-36-SIGNOFF.md` | Record the gate decision, the branch-topology change, and G5's retirement |
| `.kbd-orchestrator/phases/phase-36-sovereign-sfu-ice-linux-fix/goals.md` | Amend G5 to reflect that the PR is moot and the work is already on `main` |
