# Phase-27 Sign-off — sovereign SFU media-transport debug & the gate decision

> Date: 2026-07-09 · Closing verification for phase-27 (p27-c004). Diagnosed the WebRTC
> media-negotiation defects, fixed three, and re-ran the proof — the decode still times out, so
> `SFU_MODE=sovereign` **stays off**. Real media-path progress plus a precise next step.

## Gates (re-run at phase close — actual results)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ (exit 0) |
| `cargo check --workspace` | ✅ (exit 0) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ (exit 0) |
| `cargo test -p frf-media-str0m --lib` | ✅ 27 passed |
| `cargo test -p frf-gateway --lib` | ✅ 40 passed (39 → 40: the `local_signals` trickle-relay test) |

## Goal status (honest)

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — instrument the media path** | ✅ MET | str0m driver lifecycle `info!` logs + harness ICE-state/candidate-count diagnostics (p27-c001). |
| **G2 — fix the media-transport defect** | ◐ **THREE FIXED, decode unconfirmed** | Bidirectional trickle ICE (p27-c002, gateway `local_signals`→WS relay + harness send/apply) and RoomJoin fan-out (p27-c003) — the three statically-diagnosed defects (no ICE exchange, no WS candidate relay, no RoomJoin). Tested at the unit level; whether they make the live exchange complete is unconfirmed (below). |
| **G3 — decode + flip** | ❌ NOT PROVEN → RE-AFFIRMED OFF | The p27-c004 re-run still timed out (30s), no `framesDecoded` (`docs/PHASE-27-DECODE-RESULT.md`); the gate stays off. |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC. |

## The honest catch — diagnostics were masked

The c004 re-run reached build → boot → **gateway healthy** → **Keto view 201** → harness, then hit
a **Playwright 30s test timeout**. Critically, the c001 ICE-state diagnostics **did not surface**:
the receiver runs a 15s `connectToSovereignSfu` pre-check **then** a 20s `probeDecodedMedia` = 35s >
Playwright's 30s test timeout, so the test aborts *before* the diagnostic `expect(...)` prints
`ice=…/remoteCandidates`. A **harness-timing defect** — flagged as a risk in c003 — masked exactly
the visibility this phase built. So whether ICE now completes is still unknown.

## The gate decision — stated plainly

**`SFU_MODE=sovereign` stays off.** No decoded frame observed. `main.rs` keeps the honest gate-off
warning (verified untouched). Twelve phases, and the gate has never flipped on less than a real
decoded frame — including this phase, where the media fixes are real but the proof did not pass.

**To make it pass (next phase):** fix the harness timing (drop the redundant connect pre-check; the
probe does its own connect + RoomJoin; `test.setTimeout(60_000)`), re-run, **read the now-surfaced
diagnostics** (`ice=<state> remoteCandidates=N` + `docker compose logs gateway` str0m lifecycle),
fix whatever they reveal, and observe `framesDecoded > 0`. Only then flip.

## Process note (honest)

- The three media-negotiation fixes were **diagnosed statically first** (FIND-1/2/3), then
  implemented + unit-tested — evidence-led, not guesswork.
- The c004 run was executed for real; it failed, and the failure exposed a *second* concrete defect
  (the harness-timing bug that masked the diagnostics) — recorded plainly, not dressed up.
- **The flip-guard was confirmed, not assumed:** the guard question first came back "force the
  flip / relaxed proof"; it was re-confirmed to "keep the honest gate" before planning, so no
  forced flip entered the plan — and c004 re-affirmed gated on the real (failing) outcome.
- Both carried QA lessons applied: read the gate verdict + the archive output; **seed the openspec
  change dir at the start of every apply** (the phase-26 c002 lesson) — done for c001–c004.

## Sign-off

Phase-27 diagnosed and fixed three real WebRTC media-negotiation defects and instrumented the path,
then honestly recorded that the decode still times out (with the diagnostics masked by a
harness-timing bug), so **`SFU_MODE=sovereign` is re-affirmed off** with a precise next step. No new
CRITICAL/HIGH; all gates green. Hosted (LiveKit) remains the media path. No plane was advertised
beyond what it does.
