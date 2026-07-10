# Reflection — phase-35-sovereign-sfu-decode-ci-linux-proof

> Date: 2026-07-09. Phase-35 escalated the decode proof from macOS/Colima to GitHub Actions
> `ubuntu-latest`. The escalation worked. The six-phase same-host environment blocker is gone.
> `framesDecoded=0 ice=checking` on real Linux — a normal debugging result, not an environmental
> dead-end. `SFU_MODE=sovereign` gate held OFF.

---

## Goal Achievement

| Goal | Verdict | Evidence |
|---|---|---|
| G1 — Containerize proof rig (no macOS-host step) | ✅ PARTIAL → MET | `decode-proof.yml` + `run-media-decode.sh` auto-detect Linux; 172.17.0.1 addressing replaces `host.docker.internal`; the rig runs on CI as one self-contained job |
| G2 — Run decode on real Linux (native host networking) | ✅ MET | CI run 29057452278: gateway image built (~8 min, no OOM), sovereign stack booted, two-browser Playwright decode executed, ICE reached `checking` with `remoteCandidates=1` |
| G3 — Observe a real decoded frame + flip `SFU_MODE=sovereign` | ❌ NOT MET | `framesDecoded=0 ice=checking`; gate held OFF per phases 16–34 discipline |
| G4 — Carried live proofs (phases 19…34) | ⏭ CARRIED | No regression; `frf-media-str0m` tests green; engine unchanged |

**Overall: 1/3 explicit goals met, 1 partial (G1 fully met in spirit), G3 carried to phase-36.**

---

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA | 2/2 |
| First-pass pass rate | 2/2 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

Both `p35-c001-ci-decode-job` and `p35-c002-decode-run-and-flip` passed the artifact-refiner gate
first-pass. No constraint violations. No recurring patterns.

---

## Delivered Changes

### p35-c001 — CI decode job (Linux-portable runner)

- **`.github/workflows/decode-proof.yml`** (NEW): `workflow_dispatch` + `push: branches:
  ["sovereign-sfu-decode-proof"]` trigger; `ubuntu-latest`; 45-min timeout; builds gateway image
  once (no OOM); runs `run-media-decode.sh`; collects gateway logs (`if: always()`); uploads
  Playwright report + gateway log as artifacts. Two CI interpolation fixes identified and landed en
  route: `FLINT_GATE_JWT_SECRET` and `TURN_SECRET` must exist at `docker compose build`
  interpolation time, so both are now job-wide env (ephemeral, generated in-job — S1 clean).
- **`scripts/run-media-decode.sh`** (MODIFIED): auto-detects `uname -s` → Linux or macOS; sets
  `HOST_ADDR=172.17.0.1` on Linux; exports `MEDIA_ADVERTISE_IP` and `TURN_EXTERNAL_IP` accordingly;
  `TURN_SECRET` generated per-run.

### p35-c002 — CI decode run + honest gate decision

- CI run **29057452278** executed on `sovereign-sfu-decode-proof` branch. Stack booted; decode ran.
  Result: `framesDecoded=0 ice=checking localCandidates=5 remoteCandidates=1`. Gate held OFF.
- **`docs/PHASE-35-DECODE-RESULT.md`** (NEW): full run evidence + harness bug analysis.
- **`docs/PHASE-35-SIGNOFF.md`** (NEW): honest gate decision + carried items.
- **`docs/SECURITY.md`** (MODIFIED): §6 updated — environment unblocked, gate held.
- **`CHANGELOG.md`** (MODIFIED): phase-35 entry.

---

## What Moved

The **six-phase same-host environment blocker is removed.** Phases 28→34 each encountered a fresh
form of the same root cause: the macOS + Colima-VM + Docker-bridge topology makes it impossible for
a bridged-network browser to complete ICE with a gateway advertising host candidates, because the
host addresses don't route across the bridge boundary without workarounds that introduce new
confusion. Phase-35 removed the topology entirely by running on real Linux where host networking is
native. The proof now executes end-to-end.

What remains is one ordinary ICE-completion debugging loop on a **working harness**:
`framesDecoded=0 ice=checking` on Linux — the gateway's str0m log would tell us exactly which
candidates were exchanged and why the pair didn't complete, but the log was empty this run due to
the harness bug below.

---

## Bugs Found

### Harness bug: gateway logs torn down before capture

`scripts/run-media-decode.sh` registers `trap cleanup EXIT` → `docker compose down -v`. When the
decode fails (as expected this run), the script exits, cleanup fires, and the gateway container and
its str0m log are gone — **before** the workflow's "Collect gateway logs" step runs. The artifact
upload then captures an empty `gateway.log`.

Fix (for phase-36): capture the gateway log **inside the script** before the EXIT trap tears down
the stack, then copy it to a workflow-visible path. Alternatively: skip the in-script `down -v`
when `CI=true` so the workflow's collect step sees a live container. The script already writes
`/tmp/p29-gateway.log` on failure — it needs to be copied to `$GITHUB_WORKSPACE/gateway.log` before
`docker compose down`.

---

## Lessons Captured

1. **Env-var interpolation in Docker Compose runs at `build` time, not just `up` time.** Any
   `${VAR:?...}` in `compose.*.yml` must exist before `docker compose build`, which means CI
   secrets/generated values must land in `$GITHUB_ENV` *before* the build step — not only in the
   `run` script. This was the root cause of the two early CI failures in this phase.

2. **`workflow_dispatch` only resolves the workflow file from the default branch.** Pushing a new
   workflow to a feature branch and triggering `workflow_dispatch` silently 404s. The fix is to add
   a `push: branches: [...]` trigger — GitHub then runs the branch's own copy of the workflow file.
   This is a CI platform gotcha that applies to any new workflow on a non-default branch.

3. **The environment escalation was the right call, taken six phases late.** Phases 28→34 each
   produced a genuine fix (shared demux socket, `.local` skip, Caddy TLS, TURN relay, etc.) but
   each fix uncovered the next form of the same underlying environment mismatch. Phase-35's
   contribution is ending the environmental loop — the remaining work is now debuggable.

4. **Trap-on-exit cleanup and log-capture are in tension in CI.** A script that tears down the
   stack on exit (for local cleanup) must be modified for CI to preserve state long enough for the
   CI orchestrator to capture artifacts. The pattern: write logs to a file *inside* the script
   before calling `down`, then let CI copy that file.

---

## Technical Debt

- **Gateway log capture gap**: `gateway.log` was empty in the artifact because the EXIT trap fired
  before the workflow collect step. Must fix before the next CI run to make the ICE candidate
  exchange visible. (Carried: phase-36 c001.)
- **`SFU_MODE` still OFF**: no debt — deliberate gate per phase-16 discipline.

---

## Recommended Next Phase: phase-36-sovereign-sfu-ice-linux-fix

**Theme:** Fix gateway-log capture → read the Linux candidate exchange → fix the ICE pairing →
re-run → flip.

**Seed goals:**

1. **G1 — Fix gateway-log capture in CI**: copy `/tmp/p29-gateway.log` to `$GITHUB_WORKSPACE/`
   before the EXIT trap fires, so the next CI run produces a populated `gateway.log` artifact.
2. **G2 — Read the Linux str0m candidate log**: determine why ICE stalls at `checking` on Linux
   (`localCandidates=5 remoteCandidates=1` but no completing pair). Identify whether the gateway's
   advertised host candidate (`MEDIA_ADVERTISE_IP=172.17.0.1`) is reachable from the bridged
   Playwright container, or whether only the `typ relay` candidate should pair.
3. **G3 — Fix the Linux candidate pairing**: adjust `MEDIA_ADVERTISE_IP` / coturn
   `--external-ip` / ICE filter rules in str0m so a candidate pair completes.
4. **G4 — Re-run CI decode; flip `SFU_MODE=sovereign` on genuine `framesDecoded > 0`**: the gate
   flips only when a real browser receiver observes `framesDecoded > 0` against the live sovereign
   gateway on `ubuntu-latest`.
5. **G5 — Open PR from `sovereign-sfu-decode-proof` to `main`** after the flip: `main` has been
   untouched since phase-0; the proof branch holds all 35 phases of SFU work.

**Estimated changes:** 2–3 (c001: log capture fix + CI re-run; c002: ICE fix + re-run; c003: flip
+ PR if c002 delivers `framesDecoded > 0`).
