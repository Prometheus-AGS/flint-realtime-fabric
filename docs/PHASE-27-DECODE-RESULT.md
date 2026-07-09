# Phase-27 decoded-media proof — live run result (p27-c004)

> Date: 2026-07-09. Actual, un-fabricated outcome of the decode run after the three media-transport
> fixes (trickle ICE c002, RoomJoin fan-out c003, instrumentation c001). Input for the flip (this
> change).

## Result: ❌ NOT PROVEN — `framesDecoded > 0` not observed; diagnostics masked by a harness-timeout bug

The run again reached the media exchange (build → boot → **gateway healthy** → **Keto view 201** →
harness) and again hit a **Playwright 30s test timeout**. `RUNNER_EXIT=1`. **G3 is not met → the gate
stays off.** Two honest facts this run:

1. **The decode was not observed.** No `framesDecoded > 0`.
2. **The c001 instrumentation did not surface** — the failure is a *Playwright test timeout*, which
   aborts the test **before** the diagnostic `expect(...)` message (`ice=… remoteCandidates=…`) runs.
   So this run did not actually give us the ICE-state visibility c001 was built for.

## Root cause of the masked diagnostics — a harness-timing defect

`media-decode.spec.ts` runs the receiver's `connectToSovereignSfu` (timeout **15s**) **and then**
`probeDecodedMedia` (timeout **20s**) sequentially = up to **35s**, but Playwright's default test
timeout is **30s**. So the test is killed mid-probe, before the assertion prints the diagnostics —
and the redundant `connectToSovereignSfu` pre-check (a separate recvonly PC with no media source)
burns 15s for no decode value. (Flagged as a risk in p27-c003.)

## What is fixed vs. still unknown

- **Fixed (landed this phase):** bidirectional trickle ICE (c002 — gateway `local_signals`→WS relay
  + harness `onicecandidate`/`addIceCandidate`), RoomJoin fan-out (c003), and lifecycle
  instrumentation (c001). These are real, tested improvements.
- **Still unknown:** whether ICE now actually completes and media decodes. This run could not tell
  us because the diagnostics were masked — a harness defect, not (yet) a media-path verdict.

## Consequence for the flip (this change)

**`SFU_MODE=sovereign` stays gated OFF.** G3 unmet. c004 **re-affirms gated** with this concrete
detail — no forced flip, no relaxed proof (operator-confirmed).

## To make the proof pass (next attempt)

1. **Fix the harness timing (next change/phase):** drop the redundant `connectToSovereignSfu`
   pre-check (the probe does its own connect + RoomJoin), and raise the Playwright test timeout
   (`test.setTimeout(60_000)`) so the probe's own 20s window + the diagnostic assertion actually run.
2. Re-run; **read the now-surfaced diagnostics** — `ice=<state> localCandidates=N
   remoteCandidates=M` on the failure line, plus the gateway's str0m lifecycle logs
   (`docker compose logs gateway`: session negotiated / connection state / inbound MediaData). This
   tells us whether ICE completes (remoteCandidates>0, ice=connected) and whether the sender's RTP
   reaches the SFU/receiver.
3. Fix whatever the diagnostics reveal (candidate reachability, DTLS, fan-out over the real socket).
4. Observe `framesDecoded > 0` → then flip.
