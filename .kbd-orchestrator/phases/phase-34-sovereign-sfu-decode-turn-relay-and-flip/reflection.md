# Reflection — phase-34-sovereign-sfu-decode-turn-relay-and-flip

> Date: 2026-07-09. Phase goal: add a TURN relay to the bridge stack so a routable candidate pair
> forms, then flip `SFU_MODE=sovereign` on `framesDecoded > 0`.
> **Outcome: the TURN infrastructure is correct (str0m accepts `typ relay`, confirmed; iceServers +
> credentials wired, S1-clean) — but 0 relay candidates reached the gateway because coturn's
> `--external-ip=127.0.0.1` is the browser's own in-container loopback. `framesDecoded=0`; gate held
> OFF. This is the 11th blocker and the 6th candidate-topology form to fail on the same-host
> environment — the definitive signal to ESCALATE to CI / a real Linux host (Target B).**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — TURN relay → routable pair** | ◐ INFRA CORRECT, PAIR NOT FORMED | Exit was "the decode run forms a routable pair → `ice=connected` + `state=Connected` + `MediaData`." The TURN infra is right (str0m accepts `typ relay` — `parser.rs:232`; coturn upgraded; `iceServers` + creds in both PCs; S1-clean env secret) — but **0 `typ relay` candidates reached the gateway** because coturn advertised its relay at `--external-ip=127.0.0.1`, which in the browser's container is the browser itself. No routable pair; ICE stayed `checking → Disconnected`. NOT met. |
| **G2 — decoded frame + flip** | ❌ NOT MET (gate held) | `framesDecoded=0`; no relay pair; no `Connected`; no `MediaData`. `main.rs` untouched; `PHASE-34-DECODE-RESULT.md` records it. |
| **G3 — carried proofs** | ◐ CARRIED | Untouched, integration-gated. |

**Honest headline: 0 goals fully met.** G1's *design* was sound and de-risked in advance (str0m relay
support), but a single config value (`--external-ip`) meant no relay candidate ever formed — the
**same bridge loopback-vs-container address confusion** that has now defeated the proof in **six
forms** (host-loopback, bridge, mDNS, srflx, host-net, relay-external-ip) across phases 28→34. The
media path is proven correct in pieces every single time; the environment is the sole recurring
blocker.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p34-c001-coturn-turn-relay | coturn STUN→TURN (env-secret, S1-clean) + harness `iceServers` turn: in both PCs | none (harness); str0m relay confirmed |
| p34-c002-decode-run-and-flip | Bridge+TURN decode run + honest gate decision + CI escalation | **held OFF** |

Both archived (`openspec/changes/archive/2026-07-09-p34-c00{1,2}-*`); `media-e2e` +
`sfu-mode-consistency` specs updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 2/2 |
| First-pass pass rate | 2/2 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Both changes passed R1–R5, S1 (**no committed TURN secret** — env-required), P1 first-pass.
31/31 `frf-media-str0m` tests + fmt green (no engine change).

## What actually moved (honest)

- **str0m's relay acceptance is now confirmed in the source** (a durable finding for any future TURN
  deployment) and the TURN plumbing is correctly wired (credentials via env, S1-clean).
- **No media progress:** `framesDecoded=0`, no relay candidate reached the gateway — one config value
  short, but on the same environment that keeps producing a new address confusion each time.

## Technical debt introduced

- **None net-new in shipped code** (harness/compose only; no engine change). The TURN wiring is
  correct and reusable — it will work once run where the relay's `external-ip` is a genuinely
  browser-reachable address (i.e., a real Linux host / CI).
- No secrets committed (S1). The env-required `${TURN_SECRET:?...}` pattern is a reusable finding.

## Lessons captured

1. **De-risking the *engine* side does not de-risk the *environment* side.** I confirmed str0m accepts
   `typ relay` before coding (good) — but the failure was environmental (`--external-ip` = the
   browser's own loopback), which the source check couldn't surface. On an environment that has
   failed six candidate-topology variants, the next unknown is *always* environmental; verify the
   **candidate addresses actually route in the target topology**, not just that the protocol is
   accepted.
2. **Six is enough: the same-host Docker environment is categorically wrong for this proof.** Every
   phase from 28 has proven a media-path layer correct and then hit a fresh candidate-address
   confusion (loopback/bridge/mDNS/srflx/host-net/relay-ip). That is not a sequence of independent
   bugs — it is **one environmental unsuitability wearing different masks.** Continuing to patch the
   next mask locally is the definition of not learning. The escalation to CI (Target B) is overdue and
   now non-negotiable.
3. **The honest gate held a ninth time — and, more importantly, the *escalation* discipline finally
   fired.** The gate (framesDecoded==0 ⇒ no flip) is easy here. The harder, more valuable judgment was
   recognizing "stop the local loop" — which the phase-32/33 lessons set up and this phase executed:
   the result + reflection recommend CI, not a seventh local variant.
4. **`--external-ip` on a TURN relay must be the address the *client* can reach the relay at** — in a
   container, that is the relay's network-visible IP, never `127.0.0.1`. Reusable coturn finding.

## Recommended next phase

**`phase-35-sovereign-sfu-decode-ci-linux-proof`** — run the whole decode on a real Linux host where
host networking is native and candidate addresses simply route.

- **G1 (the escalation):** **containerize the host-side proof rig** (RS256 JWT mint + JWKS serve +
  Keto seed — currently macOS-host scripts) so the entire proof is one self-contained job, then run it
  on **`ubuntu-latest` (GitHub Actions)** or a self-hosted Linux runner. On Linux, host networking is
  native: the browser + SFU share one real stack, host candidates pair with no bridge/loopback/mDNS
  confusion. Add the CI workflow (or a `docker compose` profile the runner invokes). The gateway
  advertises its **real reachable IP**; STUN/TURN (already wired) are belt-and-suspenders.
- **G2:** on the runner, re-run; observe `ice=connected` + `state=Connected` + `MediaData` +
  `framesDecoded > 0`. **Flip `SFU_MODE=sovereign` only on a genuine decoded frame** — this is the run
  where the whole proven-in-pieces media stack finally runs end-to-end.
- **⚠️ Operator input:** which runner — GitHub Actions `ubuntu-latest` (uses CI minutes; the existing
  `ci.yml` is the natural home) or a self-hosted Linux box? Surface at assess/plan.
- **G3** carried (LiveKit x-node, admin-ui OIDC) — integration-gated.
- **Hard non-goal:** **no more same-host macOS/Colima network variants** — six have failed; the
  environment is the blocker.

**Discipline carried (16→33):** the gate flips only on decoded media proven live. Progress N/N before
next-stage commands (phase-29). Pivot when the environment is the blocker (phase-32) — **toward the
last working state (phase-33), and off the wrong environment entirely once it has failed enough
(phase-34)**.
