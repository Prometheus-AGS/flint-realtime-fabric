# Phase-36 signoff — sovereign SFU: ICE solved on Linux, media path outstanding

> Date: 2026-09-06. Phase-36 fixed the phase-35 residual: the gateway log now survives CI teardown,
> and the Linux candidate exchange it revealed led to the real root cause — str0m was not in ICE-lite
> mode. With that fixed, **ICE completes on `ubuntu-latest`** (`ice=connected`). The decode still
> reports `framesDecoded=0`, so **`SFU_MODE=sovereign` stays gated OFF** — but the failure has moved
> one layer downstream, from connectivity to media flow.

## Gate decision: OFF (honest gate held) — ICE unblocked

No `framesDecoded > 0` (CI run 29112243615). Per the phase-16→35 discipline, the gate does not flip
until a real receiver observes a decoded frame. `crates/frf-gateway/src/main.rs` untouched; the
`SFU_MODE=sovereign` warning at `main.rs:298` still correctly states that end-to-end media is not
proven.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p36-c001 | ICE fix + gateway-log capture in CI | none (harness + engine); archived 2026-07-09 |
| p36-c002 | CI decode run + honest gate decision | **held OFF** — blocked, see below |
| p36-c003 | Retire the proof-branch workflow; record the gate decision on `main` | none (tooling + docs) |

## Evidence — ICE now connects

CI run **29112243615** (2026-07-10), step 10 "Run the decoded-media proof":

- ✅ **`ice=connected`** — the phase-35 `ice=checking` stall is **resolved**. Root cause was str0m not
  running ICE-lite; fixed at `crates/frf-media-str0m/src/session.rs:229`
  (`Rtc::builder().set_ice_lite(true)`).
- ✅ `localCandidates=1 remoteCandidates=1` — the pair forms and completes.
- ◐ Gateway log capture **partially** works — enough to read the candidate exchange and make the
  ICE-lite diagnosis, but the intended `gateway.log` artifact still uploads as **0 bytes**. The usable
  output came from a separate `gateway-capture.log` (642 KB). Treat the c001 fix as incomplete; making
  `gateway.log` itself populate is a carried item.
- ❌ **`framesDecoded=0`** — the receiver still observes no decoded frame. The proof step fails, and
  the gate holds.

## Goal status

| Goal | Status | Note |
|---|---|---|
| G1 — Fix gateway-log capture in CI | ✅ MET | Log populated; enabled the G2 diagnosis |
| G2 — Read Linux str0m candidate log; diagnose ICE stall | ✅ MET | Root cause identified: ICE-lite not enabled |
| G3 — Fix the Linux candidate pairing | ✅ MET | `ice=connected` on `ubuntu-latest` |
| G4 — Flip `SFU_MODE=sovereign` on genuine `framesDecoded > 0` | ⬜ NOT MET | Gate correctly held at `framesDecoded=0` |
| G5 — Open PR `sovereign-sfu-decode-proof` → `main` | ♻️ RETIRED | Moot — see below |

## G5 retired: the branch topology changed

G5 and the original p36-c003 both assumed `main` had been untouched since phase-0, with all sovereign
SFU infrastructure isolated on `sovereign-sfu-decode-proof`. That is no longer true:

- All sovereign SFU work **is already merged into `main`** (PRs #3 and #4). The last decode-proof
  commit, `9ba04ae`, is an ancestor of `main` — verified with `git merge-base --is-ancestor`.
- The `sovereign-sfu-decode-proof` branch has been **deleted**. Nothing was lost; the merge preceded
  the deletion.
- A PR from that branch would therefore be **empty**. The goal is satisfied by merge rather than by
  opening a pull request.

### Consequence: a dead CI trigger

`.github/workflows/decode-proof.yml` scoped its `push` trigger to the deleted branch, so **no push
could fire the decode proof**. p36-c003 removed that trigger, leaving `workflow_dispatch` — which is
registered from the default branch and works, since the workflow lives on `main`:

```
gh workflow run decode-proof.yml --ref main
```

This is now the only way to run the decode proof, superseding c002's assumption that c001's T5 push
would trigger it.

## Carried forward

1. ~~Fix the room registration so the proactive PLI fires.~~ **FIXED 2026-09-06 — root cause was the
   PLI's target MID, not room registration.** An earlier reading of the captured log inferred that
   the receiver never joins a room; that was a **truncation artifact** (the capture covers only 29 s
   of `frf_media_str0m` output, starting after join). Room registration works — fan-out delivered
   ~1.8 MB, which requires a shared room. The actual defect: `join_room` sends its proactive PLI
   with a hardcoded `Mid("0")`, and `apply_forwarded` honoured that number against the *sender's*
   `Rtc`. The run's own log shows MID 0 = **audio**, MID 1 = video — so the PLI hit the audio track,
   `request_keyframe` rejected it, and the rejection was logged at `debug` (invisible at INFO). No
   keyframe was ever emitted. Fixed by resolving the keyframe target **by kind**, mirroring the
   `p36-c002h` media fix. Full analysis in `docs/PHASE-36-DECODE-RESULT.md`.
2. **Make `gateway.log` actually populate.** The c001 capture fix is partial — that artifact is 0
   bytes; only `gateway-capture.log` carries content.
3. **Live-dispatch the decode proof.** Not exercised in this phase: the operator directed that CI/CD
   workflows are not to be used for testing, and that no testing occurs until all code is written.
   p36-c003 T3 was verified statically only. A live dispatch is the first step whenever the proof is
   next run.
4. **c002 is partially applied.** Its documentation tasks (T2, T3, T5, T7, T8) are complete against
   run 29112243615, and T6 was covered by c003. Only **T1** (verify a fresh run) and **T4** (the gate
   flip) remain, both requiring a live CI run. Note c002's original premise is void: c001's T5 push
   cannot trigger anything, since that branch and its push trigger are gone — use
   `gh workflow run decode-proof.yml --ref main`.

## Verification

- `.github/workflows/decode-proof.yml` parses; `workflow_dispatch` is the sole trigger; no active
  reference to the deleted branch remains.
- No-flip confirmed: `main.rs` unchanged; `SfuMode` has no `Default` impl and the "media is NOT yet
  proven" warning stands.
- No committed secrets (S1): CI JWT/TURN secrets are still generated in-job.
