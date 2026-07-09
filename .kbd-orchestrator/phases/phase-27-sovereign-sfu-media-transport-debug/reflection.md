# Reflection — phase-27-sovereign-sfu-media-transport-debug

> Generated 2026-07-09. Diagnosed the WebRTC media-negotiation defects, fixed three, instrumented
> the path, and re-ran the proof — the decode still times out, and a harness-timing bug masked the
> new diagnostics. `SFU_MODE=sovereign` re-affirmed off.

## Delta — movement against phase goals

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — instrument the media path** | ✅ MET | str0m driver lifecycle `info!` logs + harness ICE-state/candidate diagnostics (c001). |
| **G2 — fix the media-transport defect** | ◐ THREE FIXED, decode unconfirmed | Bidirectional trickle ICE (c002, gateway `local_signals`→WS relay + harness send/apply) + RoomJoin fan-out (c003) — the three statically-diagnosed defects. Unit-tested; live effect unconfirmed (diagnostics masked). |
| **G3 — decode + flip** | ❌ NOT PROVEN → RE-AFFIRMED OFF | c004 re-run still 30s-timed-out; no `framesDecoded` (`PHASE-27-DECODE-RESULT.md`). |
| **G4 — carried** | ⏳ gated | LiveKit `realtime`; admin-ui OIDC. |

**1 MET, 1 three-fixed-but-unconfirmed, G3 not proven — real media-path progress, proof not passed.**

## Root cause — why G3 did not close (two layers)

1. The c004 re-run reached the media exchange and hit a Playwright 30s timeout — no decoded frame.
2. **The c001 diagnostics were masked:** the receiver runs `connectToSovereignSfu` (15s) **then**
   `probeDecodedMedia` (20s) = 35s > Playwright's 30s test timeout, so the test aborts *before* the
   diagnostic `expect(...)` prints `ice=…/remoteCandidates`. A **harness-timing defect** (flagged as
   a risk in c003) hid exactly the visibility this phase was built to produce — so whether the c002
   ICE fix actually completes is still unknown.

## Delivered changes (4/4 archived)

1. **c001** — media instrumentation (str0m lifecycle + harness ICE-state/candidate counts).
2. **c002** — bidirectional trickle ICE over WS (FIND-1/2), + bridge `local_signals` method + test.
3. **c003** — RoomJoin fan-out (FIND-3); completed the sender's inline ICE trickle too.
4. **c004** — decode re-run (timed out, recorded) + re-affirm gate off + SECURITY §6 + CHANGELOG +
   PHASE-27-SIGNOFF.

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes | 4/4 archived |
| Final QA verdict | 4/4 ALL PASS |
| Re-runs / archive aborts | 0 |
| Openspec seed misses | 0 (seeded every change dir up front — phase-26 c002 lesson applied) |
| Release gate at close | fmt ✅ · clippy --workspace ✅ · check ✅ · str0m 27 · gateway 40 (39→40: local_signals test) |

### Notable

- **The three fixes were diagnosed statically first** (FIND-1/2/3), then implemented + unit-tested —
  evidence-led. The `local_signals`→WS relay is real, tested gateway code, not just a harness tweak.
- **The c004 run exposed a second concrete defect** (the harness-timing bug) — the most useful thing
  it produced, recorded plainly.
- The pipeline-enforce hook correctly blocked a premature reflect-mention until progress hit 4/4;
  split into separate commands (known behaviour).

## Technical debt introduced

- **None structural.** The harness-timing bug is *pre-existing* (the connect+probe sequence predates
  this phase; c003 flagged it) and now documented with a precise fix. The trickle-ICE + RoomJoin code
  is production-shaped and tested.

## Lessons captured

1. **Instrument, but make sure the instrument can report.** c001's diagnostics were correct but
   unreachable because a *test-harness timeout* fired first. A diagnostic that can't print is no
   diagnostic — the harness must guarantee the failure path surfaces it (raise the outer timeout, or
   settle-with-diagnostics before the framework kills the test).
2. **Don't stack sequential timeouts under a fixed outer budget.** 15s + 20s under a 30s Playwright
   timeout is a latent abort. Sum inner timeouts must stay under the outer, with margin.
3. **A redundant pre-check has a cost.** `connectToSovereignSfu` added no decode value (separate PC,
   no media) but consumed 15s of the budget. Cut steps that don't advance the assertion.
4. **Static diagnosis pays off.** FIND-1/2/3 were found by reading the code before running — the run
   then only needed to confirm, and the fixes were targeted rather than trial-and-error.

## Recommended next phase

**`phase-28-sovereign-sfu-decode-harness-timing-and-proof`** — the narrow finish that finally
surfaces the truth:

- **Fix the harness timing (G1):** drop the redundant `connectToSovereignSfu` pre-check (the probe
  self-connects + RoomJoins); `test.setTimeout(60_000)`; ensure the probe settles-with-diagnostics
  before any framework timeout so `ice=<state> localCandidates=N remoteCandidates=M` always prints.
- **Re-run + read the diagnostics (G2):** the failure line + `docker compose logs gateway` (str0m
  lifecycle: session negotiated / connection state / inbound MediaData) tell us whether the c002 ICE
  fix completes (remoteCandidates>0, ice=connected) and whether the sender's RTP reaches the receiver.
- **Fix whatever the diagnostics reveal, then observe `framesDecoded > 0` → flip** (main.rs +
  SECURITY §6 + CHANGELOG). Only on a genuine frame.
- Fold in G4 (LiveKit `realtime`; admin-ui OIDC).

If the surfaced diagnostics reveal a deep media-path issue (str0m ICE over mapped UDP, DTLS), ADR it
and keep the gate off — do not force it.
