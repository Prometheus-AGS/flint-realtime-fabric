# Reflection — phase-28-sovereign-sfu-decode-harness-timing-and-proof

> Date: 2026-07-09. Narrow phase: surface the decode harness's diagnostics, read the evidence, fix
> what it reveals, and flip `SFU_MODE=sovereign` **only** on a real `framesDecoded > 0`.
> **Outcome: the phase did exactly what it set out to procedurally — the truth surfaced — but the
> goal it was hunting (a decoded frame) did NOT occur. The gate stays OFF, honestly.**

## Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| **G1** — Fix the harness so diagnostics surface | ✅ MET | p28-c001 dropped the 15s connect pre-check, set `test.setTimeout(60_000)`, and dumps gateway logs on failure. The failed run now prints `framesDecoded=0 … ice=new localCandidates=2 remoteCandidates=0` — the diagnostic that was masked in phase-27 now prints. |
| **G2** — Read diagnostics + fix the revealed issue | ◐ PARTIAL | p28-c002 read the surfaced evidence, root-caused the `0.0.0.0` ICE candidate-IP bug, and **fixed it** (`advertise_host: Option<String>` + `resolve_advertised_ip`; gateway now advertises a valid `127.0.0.1:40000` host candidate). That fix is real and verified. But the re-run then surfaced **two further** blockers (B1 mDNS, B2 shared socket) that were not fixed this phase — G2's "fix whatever the evidence reveals" is one layer deep, not exhausted. G2's exit ("the concrete blocker is diagnosed and recorded, gate held") **is** satisfied. |
| **G3** — Observe a real decoded frame + flip | ❌ NOT MET (gate correctly held) | `framesDecoded == 0`. Per the operator-confirmed discipline, `main.rs` is **untouched**, SECURITY §6 keeps the plane *composed but not proven*, and `docs/PHASE-28-DECODE-RESULT.md` records the run. G3's exit ("moves real decoded media, **or** stays gated with rationale") is satisfied by the honest-hold branch. |
| **G4** — Carried live proofs (LiveKit x-node, admin-ui OIDC) | ◐ CARRIED | Not touched this phase (correctly — narrow scope). Remain integration-gated as in phase-19…27. |

**Honest headline:** 1 of 3 media goals fully MET (G1); G2 partial (fixed one of three blockers);
**G3 — the actual objective — NOT met.** The phase's *value* was surfacing + fixing the candidate-IP
bug and cleanly identifying the two remaining blockers, not reaching a decoded frame. Calling this a
"success" would be sycophantic: the plane still does not move decoded media, and that is the metric.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p28-c001-harness-visibility | Drop connect pre-check; 60s timeout; gateway-log dump on failure | none (harness) |
| p28-c002-diagnose-and-fix | `0.0.0.0`→resolvable `advertise_host` ICE candidate-IP fix; `resolve_advertised_ip` | none (adapter); real fix |
| p28-c003-decode-run-and-flip | Decode re-run; honest gate decision (OFF); DECODE-RESULT + SIGNOFF + SECURITY §6 + CHANGELOG | **held OFF** |

All three archived (`openspec/changes/archive/2026-07-09-p28-c00{1,2,3}-*`); `sfu-mode-consistency`
spec updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 3/3 |
| First-pass pass rate | 3/3 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Every change passed R1 (check), R2/R3 (clippy pedantic + unwrap_used), R4 (fmt), R5 (≤500
lines), S1 (no secret), P1 (openspec validate) on the first pass. 29/29 `frf-media-str0m` lib tests
green.

## What actually moved (honest progress, not spin)

- The gateway-side ICE candidate is now **valid** — every prior run failed *before* the media
  exchange; this phase's run reached it (`session negotiated … advertised=127.0.0.1 40000 typ host`).
- The remaining unknowns collapsed from "why does decode time out?" to **two named, well-understood
  str0m-SFU engineering items**. That is the real deliverable.

## Technical debt introduced

- **None net-new in shipped code.** The two blockers (B1, B2) are pre-existing limitations of the
  in-process-loopback-only design, now *exposed* rather than *introduced* — the loopback proof never
  exercised a real browser's mDNS candidates or a two-peer room on a single fixed port.
- The decode harness carries run-specific scaffolding (self-mint RS256 JWKS, Keto seed, compose
  overrides) — acceptable for a proof runner, not production wiring; flagged for eventual cleanup.

## Lessons captured

1. **A single "fix the revealed bug" phase rarely exhausts the bug list on a real integration.**
   Fixing the `0.0.0.0` candidate simply let the negotiation proceed far enough to reveal the *next*
   two issues. Surfacing diagnostics (G1) was the true leverage — it converts one guess-fix loop
   into an evidence-driven sequence. Budget for "fix → new evidence → fix" chains, not a single shot.
2. **Loopback proofs hide the two hardest browser↔SFU realities:** mDNS host candidates and
   single-socket demuxing. Both are invisible until a real `RTCPeerConnection` connects. The
   in-process `two_peers_reach_dtls_connected` test is necessary but *not* sufficient — it will stay
   green while B1/B2 keep real decode at zero.
3. **The honest gate held under a third re-confirm-by-omission.** No operator pressure this phase, so
   the discipline ran on its own: `framesDecoded==0 ⇒ no flip`, no rationalization from "but the
   candidate is valid now." Progress ≠ proof.
4. **Seed-dir-up-front + read-archive-output both paid off** — c003 archived cleanly on the direct
   `openspec archive` after the wrapper returned RC=1 silently; reading the actual archive output
   (not the wrapper's exit code) caught it. Carry: **never trust a silent archive.**

## Recommended next phase

**`phase-29-sovereign-sfu-shared-socket-and-stun`** — clear B2 then B1, then re-run the decode proof.

- **B2 (do first — it blocks everything):** replace the per-session fixed-UDP-port bind in
  `StrOmTransport` with **one shared demuxing UDP socket**; route each inbound datagram to the owning
  `Rtc` by ICE ufrag / remote 5-tuple (str0m's own examples do exactly this). ADR the transport
  change (touches ADR-005/006 socket ownership). Kills the `EADDRINUSE`/"unknown session" cascade.
- **B1:** give the browser a routable candidate the gateway accepts — a **STUN server-reflexive
  path** (add a STUN server so Chrome emits an srflx candidate) or **mDNS `.local` resolution** at
  the gateway. The browser host `.local` candidate alone is not usable.
- Then re-run `scripts/run-media-decode.sh`; **flip `SFU_MODE=sovereign` only on a genuine
  `framesDecoded > 0`.** Same discipline, same gate.
- **G4** carried unchanged (LiveKit x-node, admin-ui OIDC) — integration-gated.

**Discipline carried (16→28):** the gate flips only on decoded media proven against a live gateway;
else it stays off with fresh rationale. No "healthy but does nothing."
