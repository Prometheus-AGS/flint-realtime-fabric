# Reflection — phase-33-sovereign-sfu-decode-real-network-proof

> Date: 2026-07-09. Phase goal: pivot the decode proof to an environment where the browser + SFU
> share a routable network (Target A: local host-net), so a candidate pair forms, then flip
> `SFU_MODE=sovereign` on `framesDecoded > 0`.
> **Outcome: Target A was structurally implemented but hit its OWN Colima plumbing failure — the
> gateway goes `unhealthy` under `network_mode: host` — so the decode never ran. `framesDecoded=0`;
> gate held OFF. This phase made NO forward progress toward a decoded frame and in fact regressed the
> live signal vs. phase-32 (which reached ICE `checking`), for an infrastructure reason. The honest
> conclusion: Target A is a dead end on this box; pivot to TURN (Target C).**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — routable-network environment** | ❌ NOT MET | Exit was "the decode run reaches `ice=connected` + ≥1 session `state=Connected` + inbound `MediaData`." The host-net override is **structurally correct** (`!reset` cleared merge-inherited `ports:`; Colima IPs derived; VM→host JWKS reachability confirmed) — but the **gateway container goes `unhealthy` under `network_mode: host`** (`/readyz`/`8080` interaction, not JWKS, not media), cascading to the browser via `depends_on`. The decode **never ran**; ICE never started. NOT met. |
| **G2 — TURN relay** | ⊘ NOT ATTEMPTED | Target A does not use TURN (host candidates were meant to pair directly). G2 was the belt-and-suspenders; it became the **recommended pivot** instead of a delivered exit. |
| **G3 — decoded frame + flip** | ❌ NOT MET (gate held) | `framesDecoded=0`; gateway never healthy; no session. `main.rs` untouched. Honest-hold branch satisfied; objective did not occur. |
| **G4 — carried proofs** | ◐ CARRIED | Untouched, integration-gated. |

**Honest headline: 0 goals met, and a *regression* in live signal.** Phase-32 reached ICE `checking`
(the media exchange ran); phase-33's host-net attempt didn't even get the gateway healthy, so the
exchange never started. Target A — the "lowest-effort" bet — cost a phase and moved *backward* on the
live signal, because it swapped the (working) bridge stack for a host-net stack that fails its own
health check on Colima. That is the honest assessment: **a wrong turn, correctly abandoned.**

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p33-c001-host-net-decode-stack | `compose.host-net.yml` (`network_mode: host` via `!reset`) + runner `HOST_NET=1` + Colima IP derivation | none (harness); structurally correct but the target is a dead end |
| p33-c002-decode-run-and-flip | Host-net decode run + honest gate decision + TURN-pivot recommendation | **held OFF** |

Both archived (`openspec/changes/archive/2026-07-09-p33-c00{1,2}-*`); `media-e2e` +
`sfu-mode-consistency` specs updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 2/2 |
| First-pass pass rate | 2/2 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Both changes passed R1–R5, S1, P1 first-pass. 31/31 `frf-media-str0m` tests + fmt green (no
engine change). **Note:** clean QA on the *artifacts* does not mean the phase advanced the goal — it
did not. Artifact quality ≠ goal progress.

## What actually moved (honestly: essentially nothing toward the goal)

- The host-net override is a correct, reusable artifact (the `!reset` merge-trap fix + Colima IP
  derivation are genuinely useful if host-net is ever wanted).
- But **no media progress** — the decode never ran; the live signal regressed from phase-32's ICE
  `checking` to "gateway unhealthy, nothing started."

## Technical debt introduced

- **`compose.host-net.yml` is now a dead-end artifact** — it works structurally but the target fails on
  this environment. Keep it (documents the attempt + the `!reset` pattern) but do not build on it;
  mark it superseded by the TURN path.
- No net-new shipped-code debt (harness/compose only; no engine change).

## Lessons captured

1. **"Lowest-effort" is not "lowest-risk" when it swaps a working substrate.** Target A was picked as
   the cheap test, but it *replaced* the bridge stack (which at least reached ICE `checking`) with a
   host-net stack that fails its own health check. Choosing the option that discards a known-partially-
   working path is how a phase regresses. When a prior phase reached further, the next step should
   **build on that stack**, not swap it for an untested one — which is exactly what **TURN on the
   bridge (Target C)** does.
2. **I should have weighted the operator options by "distance from the last working state," not just
   raw effort.** Phase-32's bridge+HTTPS stack got the furthest; C (TURN on that same stack) is the
   true minimal next step. A (host-net) was a lateral move that reset progress. Recording this so the
   next environment decision ranks by "closest to the furthest-working state."
3. **The honest gate held an eighth time — trivially, because there was nothing to tempt it.** No
   decoded frame, no session, no near-miss. The discipline's harder test is a near-miss (phase-32);
   this was an easy hold. Worth noting the gate isn't the only guardrail — the *goal-progress* honesty
   (calling this a regression) is the one that mattered this phase.
4. **`!reset` (Compose 2.24+) is the correct tool to remove merge-inherited list entries** (`ports:`,
   `extra_hosts:`) in a layered override — a real, reusable finding regardless of the host-net verdict.

## Recommended next phase

**`phase-34-sovereign-sfu-decode-turn-relay-and-flip`** — the true minimal next step: add TURN to the
**bridge** stack that already reached ICE `checking` (phase-32), so a relay candidate always routes.

- **G1 (primary):** on the **phase-32 bridge + HTTPS stack** (NOT host-net), upgrade coturn from
  STUN-only to a **TURN relay** — `--external-ip=<gateway/bridge IP>` (or `--relay-ip`), a realm, and
  long-term credentials. Add the `turn:` URL (+ username/credential) to the harness `iceServers`
  alongside STUN. Advertise the gateway's **bridge IP** via `MEDIA_ADVERTISE_IP`. This is the standard
  WebRTC answer to "no routable host/srflx pair": a **relay candidate routes regardless of topology**.
  Verify str0m accepts + pairs the relay candidate.
- **G2:** re-run the decode on the bridge+TURN stack; observe `ice=connected` + `state=Connected` +
  `MediaData` + `framesDecoded > 0`. **Flip `SFU_MODE=sovereign` only on a genuine decoded frame.**
- **G3:** if TURN on the bridge *also* fails, that is the strong signal to move to **CI / a real Linux
  host (Target B)** — do not attempt further local network variants.
- **G4** carried (LiveKit x-node, admin-ui OIDC) — integration-gated.

**Discipline carried (16→32):** the gate flips only on decoded media proven live; else it stays off
with fresh rationale. Progress N/N before next-stage commands (phase-29). Absence of a defect is not
presence of a proof (phase-30). Pivot when the environment is the blocker (phase-32) — **and pivot
toward the last working state, not away from it (phase-33)**.
