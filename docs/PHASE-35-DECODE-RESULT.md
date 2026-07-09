# Phase-35 decode result (p35-c002)

> Date: 2026-07-09. The decode proof ran **on GitHub Actions `ubuntu-latest`** (branch
> `sovereign-sfu-decode-proof`, run 29057452278) — the environment escalation the last four phases
> pointed to. **The escalation succeeded: for the first time the whole stack builds + boots + runs the
> two-browser decode end-to-end on real Linux.** The decode itself is `framesDecoded=0` at
> `ice=checking` — a genuine ICE-completion result (not an environment/tooling failure), plus one
> harness bug (gateway logs torn down before capture). **`SFU_MODE=sovereign` stays gated OFF.**

## Outcome: NOT proven — gate stays OFF (framesDecoded=0)

```
framesDecoded=0 bytes=0 reason=timeout ice=checking localCandidates=5 remoteCandidates=1
1 failed (2 retries)  ·  job conclusion: failure
```

## The escalation WORKED — the environment is no longer the blocker

The CI job (p35-c001) ran the full proof on `ubuntu-latest`, and every setup + boot layer passed
(after two trivial CI-config fixes: `FLINT_GATE_JWT_SECRET` and `TURN_SECRET` had to exist at
`docker compose build` interpolation time, now generated job-wide):

- ✅ **Gateway image built on CI** (~8 min, no OOM — the phase-30 Colima OOM was VM contention, proven
  again).
- ✅ **The sovereign stack booted and the two-browser Playwright decode ran** — `getUserMedia`
  succeeded, the WS signaling + offer happened, ICE reached **`checking`** with `remoteCandidates=1`.

So the six-phase environment blocker (the same-host macOS/Colima candidate-topology confusion) is
**removed** — the proof now executes on a real Linux host with native host networking.

## The remaining media issue: ICE stalls at `checking` on Linux too

`framesDecoded=0`, `ice=checking`, `remoteCandidates=1` — the same "no completing candidate pair" shape
seen locally, now on Linux. But the **gateway str0m log is empty in the artifact** (`gateway.log` = 0
bytes), so the gateway-side candidate detail (host vs. `.local` vs. `typ relay`) is not visible for
this run — because of a harness bug (below). So the Linux root cause is **not yet diagnosable** from
this run's evidence; two concrete fixes are needed before the next run tells us.

### Harness bug found: gateway logs torn down before capture

`scripts/run-media-decode.sh` has `trap cleanup EXIT` → `docker compose down -v`, which runs when the
script exits (the decode failed) **before** the workflow's "Collect gateway logs" step — so the
gateway container (and its str0m log) is gone by the time the workflow reads it. The 725 KB artifact
has the full Playwright trace/report but an **empty `gateway.log`**. Fix: capture the gateway log
**inside** the script before the cleanup trap fires (the script already writes `/tmp/p29-gateway.log`
on failure — it must be copied to a workflow-visible path/artifact), or disable the in-script
`down -v` in CI and let the job collect logs first.

### Likely Linux candidate-addressing (to verify next run)

On Linux the runner sets `MEDIA_ADVERTISE_IP=172.17.0.1` (docker0 gateway) and `TURN_EXTERNAL_IP` the
same. The in-network browser (bridge container) should reach `172.17.0.1`, but whether the gateway's
advertised host candidate + the coturn relay candidate actually pair needs the gateway log to confirm
— hence fixing log capture first is the prerequisite.

## Decision — honest gate held (real progress: environment unblocked)

`framesDecoded == 0`. **`SFU_MODE=sovereign` is NOT flipped.** `main.rs` untouched; SECURITY §6 keeps
the plane *composed but not proven*. **But this is materially different from phases 28→34:** the
environment is no longer the blocker — the proof runs end-to-end on Linux CI. What remains is (a) a
one-line harness fix to actually capture the gateway str0m log, then (b) reading it to fix the Linux
candidate pairing. This is a normal debugging loop on a working proof harness, not another
environmental dead-end.

## Next (carried)

1. **Fix gateway-log capture in CI** (copy the in-script `/tmp/p29-gateway.log` to a workflow artifact
   path, or skip the in-script `down -v` when `CI=true` so the workflow's collect step sees a live
   container).
2. **Re-run; read the gateway str0m log** — see whether host / relay candidates reach the gateway and
   why the pair doesn't complete; adjust `MEDIA_ADVERTISE_IP`/coturn `--external-ip` to the runner's
   actual reachable address if needed.
3. Flip `SFU_MODE=sovereign` only on a genuine `framesDecoded > 0`.

Run + full artifacts: GitHub Actions run **29057452278** on branch `sovereign-sfu-decode-proof`
(`main` untouched).
