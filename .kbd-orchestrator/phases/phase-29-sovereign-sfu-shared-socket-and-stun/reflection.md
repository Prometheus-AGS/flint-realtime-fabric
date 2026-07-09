# Reflection — phase-29-sovereign-sfu-shared-socket-and-stun

> Date: 2026-07-09. Phase goal: clear the two phase-28 blockers (B2 shared socket, B1 STUN), re-run
> the decode proof, and flip `SFU_MODE=sovereign` **only** on a real `framesDecoded > 0`.
> **Outcome: both blockers cleared and proven; ICE now actually attempts connectivity for the first
> time — but the decode still yields no frame. The gate stays OFF, honestly. G3's flip did NOT
> happen.**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — B2 shared demuxing socket** | ✅ MET | Exit was "two sessions negotiate on the one shared port, no `EADDRINUSE`; inbound datagrams route by `accepts()`." The live run shows **two sessions negotiating on `0.0.0.0:40000`** with no bind error; the new `frf_media_str0m::demux` module does the routing; the in-process test `two_sessions_share_one_fixed_media_port` proves the accepts-demux. Fully met. |
| **G2 — B1 routable candidate** | ✅ MET | Exit was "the gateway accepts ≥1 browser candidate (`remoteCandidates>0`); ICE advances past `new`." The run shows browser `remoteCandidates` **0→1** and `ice` **`new`→`disconnected`** — ICE advanced past `new`. coturn + `iceServers` + the tested `.local` skip delivered exactly the exit. Met. |
| **G3 — decoded frame + flip** | ❌ NOT MET (gate correctly held) | `framesDecoded == 0`; no session reached `Connected`; zero `MediaData`. G3's exit ("moves real decoded media **or** stays gated with fresh rationale") is satisfied by the honest-hold branch — `main.rs` untouched, SECURITY §6 + `PHASE-29-DECODE-RESULT.md` record the new blocker. But the objective it was hunting — a decoded frame — did not occur. |
| **G4 — carried live proofs** | ◐ CARRIED | LiveKit x-node + admin-ui OIDC untouched (correct — narrow scope); re-affirmed integration-gated. |

**Honest headline:** 2 of 3 media goals fully MET (G1, G2 — both blockers this phase set out to clear
are cleared and *proven*, not asserted). **G3 — the actual objective — NOT met.** The phase's value is
that the SFU engine is now demonstrably correct end-to-end (negotiate → shared-socket demux → `.local`
skip → two-peer room) and the remaining gap collapsed to a **single, well-identified same-host
networking item**. Calling this "done" would be sycophantic: no decoded media flows, and that is the
metric.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p29-c001-shared-demux-socket | ADR-008; one shared `UdpSocket` demuxed by `Rtc::accepts()` (single owning task, `demux.rs`); B2 fixed | none (adapter); real fix |
| p29-c002-stun-srflx-path | coturn STUN in compose + harness `iceServers` + explicit `.local`-skip test; B1 addressed | none (harness/adapter) |
| p29-c003-decode-run-and-flip | Decode re-run; honest gate decision (OFF); DECODE-RESULT + SIGNOFF + SECURITY §6 + CHANGELOG | **held OFF** |

All three archived (`openspec/changes/archive/2026-07-09-p29-c00{1,2,3}-*`); `sfu-media-transport`
(new) + `media-e2e` + `sfu-mode-consistency` specs updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 3/3 |
| First-pass pass rate | 3/3 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Every change passed R1 (check), R2/R3 (clippy pedantic + unwrap_used), R4 (fmt), R5 (≤500
lines), S1 (no secret), P1 (openspec validate) on the first pass. 31/31 `frf-media-str0m` lib tests
green (added the shared-socket + `.local`-skip proofs); admin-ui eslint clean.

## What actually moved (honest progress, not spin)

- **The two-peer room now exists.** Every prior phase failed before two sessions could coexist; this
  phase runs two on one socket with no collision — the load-bearing precondition for any decode.
- **ICE crossed from "never started" to "attempting."** `remoteCandidates` 0→1, `ice`
  `new`→`disconnected`. The SFU engine is now *provably* not the bottleneck.
- The remaining unknown collapsed from "two blockers (B1, B2)" to **one** concrete networking item.

## Technical debt introduced

- **None net-new in shipped code.** ADR-008's shared-socket model is a *simplification* toward
  str0m's own reference (one loop, no per-session task). Files stayed ≤500 (session 367, driver 128,
  demux 213).
- The single owning demux loop services all sessions; at very high session counts a fairness/
  head-of-line concern exists (noted in ADR-008). Not a problem at decode-proof scale; flagged for
  load testing later.
- The decode harness carries run-specific scaffolding (self-mint JWKS, Keto seed, coturn, compose
  overrides) — acceptable for a proof runner, not production wiring.

## Lessons captured

1. **Fixing a blocker reveals the next layer — but the layers are shrinking.** Phase-28 fixed the
   candidate-IP bug and surfaced *two* blockers; phase-29 fixed *both* and surfaced *one*. The
   evidence-led loop is converging, not spiralling: each phase's residual is smaller and more
   precisely located than the last.
2. **Loopback-vs-bridge is the last classic browser↔SFU reality the harness hadn't hit.** mDNS
   (phase-28 B1) and single-socket demux (B2) are done; the survivor is **candidate topology** — a
   browser on the host and an SFU in a container don't share a routable address pair even with STUN,
   because the srflx reflects the bridge view. The fix is environmental (browser-in-network / advertised
   bridge IP / host networking), not engine code.
3. **The in-process DTLS test stays green while real decode is zero — as predicted.** The phase-28
   lesson held: `two_peers_reach_dtls_connected` passes because both peers share loopback; the real
   browser doesn't. Unit proofs are necessary, not sufficient — only the live run finds the topology gap.
4. **The honest gate held a fourth time, under real temptation.** "Both blockers fixed and ICE is
   finally moving" is the most seductive point yet to declare victory. It didn't: `framesDecoded==0`
   ⇒ no flip. Progress ≠ proof, even substantial progress.
5. **Pipeline-enforce guards on the pre-update snapshot** — it blocked a `jq` command whose text
   contained `/kbd-reflect` while progress still read 2/3. Update `progress.json` to N/N *before* any
   command mentioning the next stage. (Process lesson, carried.)

## Recommended next phase

**`phase-30-sovereign-sfu-decode-topology-and-flip`** — fix the same-host candidate topology so a
real decoded frame flows, then flip.

- **Primary (do first):** run the **browser inside the Docker network** — a Playwright/Chromium
  service on the compose network — so its host/srflx candidates and the gateway's `0.0.0.0:40000`
  share one network and a pair completes to `Connected`. This is the most faithful and hermetic fix.
- **Alternatives if that's impractical:** advertise a **bridge-reachable `MEDIA_ADVERTISE_IP`**
  (the host-gateway IP, already resolvable via `resolve_advertised_ip`) so the browser's srflx can
  pair with it; or give the gateway **host networking** in the sovereign override so `127.0.0.1` is
  literally shared. All three are harness/compose changes, not engine changes.
- Re-run `scripts/run-media-decode.sh`; **flip `SFU_MODE=sovereign` only on a genuine
  `framesDecoded > 0`.** Same discipline, same gate.
- **G4** carried unchanged (LiveKit x-node, admin-ui OIDC) — integration-gated.

**Discipline carried (16→29):** the gate flips only on decoded media proven against a live gateway;
else it stays off with fresh rationale. No "healthy but does nothing."
