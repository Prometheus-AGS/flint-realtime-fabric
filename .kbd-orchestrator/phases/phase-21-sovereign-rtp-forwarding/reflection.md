# Reflection — phase-21-sovereign-rtp-forwarding

> Generated: 2026-07-08
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **5 / 5 DONE**, all archived. Seeded from the phase-20 reflection.

## Summary

Phase-21 built the sovereign SFU **media path** on the phase-20 connected engine: a real
two-peer DTLS-connected proof and 1-to-1 RTP forwarding. The assessment's two decisive acts
were (1) flagging that `session.rs` was at 498/500 and had to be **split first** (a refactor,
not a feature), and (2) naming the load-bearing unknown — RTP forwarding across **isolated
per-session driver tasks** is cross-task message passing, so it needed an ADR (ADR-006:
central `RoomRouter` + per-session forwarding `mpsc`). The phase honored the honest bar
throughout: the two-peer connectivity is a *real* in-process proof (not `#[ignore]`d), the
forwarding is layer-proven (router unit + wiring integration), and — critically —
`SFU_MODE=sovereign` **stays gated off** because the browser end-to-end proof (two connected
peers exchange *decoded* media) was not stood up. The one blemish was a process miss: c004 was
archived on a QA-gate BLOCK before I read the verdict; caught, fixed, re-verified to ALL PASS.

## Goal Achievement (against the operator-scoped plan)

| Goal | Intent | Delivered | Verdict | Evidence |
|------|--------|-----------|---------|----------|
| **G0** | split session.rs (498/500) | `driver.rs` extracted, behavior-preserving | **MET** | c001; session.rs 498→377, 18 tests unchanged |
| **G1** | offerer role + connected proof | two peers reach DTLS-connected in-process | **MET (real proof)** | c002; `two_peers_reach_dtls_connected` un-ignored, passes |
| **G2 arch** | RTP fan-out decision | ADR-006: central registry + per-session mpsc | **MET** | c003; recommend + pt/back-pressure caveats |
| **G2** | 1-to-1 RTP forwarding | `RoomRouter` + driver forward arm + `join_room` | **MET (layer-proven)** | c004; router unit + wiring integration tests |
| **G3/G4** | N-peer/PLI + gate flip | **deferred to phase-22**; gate stays off | **CORRECTLY DEFERRED** | c005; browser end-to-end proof not done |
| **G5** | live cross-node / OIDC | re-affirmed carried | **MET (re-affirmed deferred)** | c005 |
| **close** | seed + sign-off | phase-22 seeded; §6/CHANGELOG/sign-off; gates green | **MET** | c005 |

**Score: 5 / 5 delivered** against the operator-set scope. G3/G4 (end-to-end media + the gate
flip) were a *deliberate deferral*, not a shortfall — the engine is present + layer-proven; the
browser proof + N-peer/PLI are phase-22. `SFU_MODE=sovereign` moves no end-to-end media.

## Delivered Changes (5)

- **Refactor-first:** c001 (session/driver split — cleared the 498/500 blocker).
- **Proof:** c002 (two-peer DTLS-connected, real in-process, un-`#[ignore]`d).
- **Decision:** c003 (ADR-006 — the cross-task RTP fan-out architecture).
- **Media:** c004 (1-to-1 RTP forwarding — `RoomRouter` + driver + wiring).
- **Close:** c005 (phase-22 seed + §6/CHANGELOG/sign-off; gate kept off).

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA gate run | 3 (c001, c002, c004; c003 ADR + c005 close docs-only, skipped) |
| First-pass gate pass rate | **2 / 3** |
| Changes that BLOCKED then passed | 1 (c004) |
| Changes requiring code refinement after a gate | 1 (c004) |

### Recurring patterns

- **The one BLOCK (c004) was a *self-inflicted process miss*, not a hard bug.** The QA gate
  flagged clippy `match_same_arms` (two empty arms in `RoomRouter::forward`); my incremental
  per-crate `--lib`/`--tests` clippy had **cached past it**, and I **archived before reading
  the gate verdict**. Root cause: I trusted the `verify` step + my own clippy over the gate's
  clean-workspace clippy. Corrective action: collapsed the match, re-ran the gate to a genuine
  ALL PASS. **Lesson (new): the QA-gate verdict is authoritative — read it before archiving;
  the gate's `--workspace` clippy from clean catches what a cached per-crate split misses.**
- **str0m API grounded from vendored source repeatedly** — `sdp_api().add_media().apply()` +
  `accept_answer` (offerer), `Candidate::from_sdp_string`, `Mid::from`/`Pt::new_with_value`/
  `MediaTime::ZERO`, `writer(mid).write(pt, network_time, time, data)`, `is_connected()`.
  Reading the crate avoided guess-and-compile churn on unfamiliar media APIs.
- **The c001 split (done first) paid off immediately** — c004 added `room.rs` + grew
  `session.rs`/`driver.rs`; without the headroom the first forwarding edit would have breached
  500. Refactor-for-headroom before feature work was the right order.

## Operational note

Mid-phase the disk hit **100% full** (`target/debug` had grown to 58 GB across phases 16–21);
`cargo clean` freed 64 GiB. Afterward every heavy gate was a cold rebuild, so I ran
clippy/tests/QA-gate **in the background** to survive the 2-minute tool timeout — the pattern
that let the phase finish. Worth carrying: background the workspace gates on this repo.

## Technical Debt Introduced / Carried

1. **End-to-end browser media proof + N-peer fan-out + PLI** — deferred to phase-22 (by
   design). 1-to-1 forwarding is wired + layer-proven; two-connected-peers-exchange-decoded-
   media is not. `SFU_MODE=sovereign` stays gated off.
2. **Gateway composition not wired** — `StrOmTransport` implements `MediaTransport` but the
   gateway's `SFU_MODE=sovereign` branch still uses the signaling-only `StrOmSignaler` and
   warns "no media." Composing both ports + flipping the gate is phase-22 G3/G4.
3. **`pt` remap not handled** — c004 forwards 1-to-1 assuming both peers negotiated the same
   codec (ADR-006 caveat); general payload-type remapping is a follow-on.
4. **LiveKit live proof + admin-ui OIDC** — carried (external-infra gated).

## Lessons Captured

- **Read the QA-gate verdict, not just `verify`, before archiving.** c004's BLOCK slipped
  through because I checked the wrong signal. The gate's clean-workspace clippy is
  authoritative over a cached per-crate run.
- **Refactor for headroom before the feature that needs it.** Splitting `session.rs` (c001)
  first is why c004's registry fit under the 500-line limit.
- **An ADR for a cross-task concern, before the code.** RTP forwarding looked like a one-line
  `writer.write`, but the real problem was moving `MediaData` between isolated driver tasks —
  ADR-006 settled the registry + channel shape before implementation (cf. ADR-004/005).
- **A "proof" can be a real in-process test even when the full thing is browser-gated.** c002
  drove str0m's actual ICE+DTLS over loopback to `Connected` — un-gating phase-20's `#[ignore]`
  honestly, without a browser.
- **Background the heavy gates on a cold build.** After `cargo clean`, foreground gates blew
  the tool timeout; `run_in_background` + a completion notification kept the loop moving.
- **Bump progress to N/N before any reflect-mentioning command** (recurring 16–21) — the
  pipeline-enforce hook still blocks otherwise.

## Recommended Next Phase

**phase-22-sovereign-sfu-npeer-pli** (already seeded in c005) — finish the sovereign media
plane to shippable:

1. **N-peer per-room fan-out** — prove 3+ peers each receive the others' media on the c004
   `RoomRouter`; simulcast/RID if needed.
2. **PLI / keyframe + renegotiation** — a late-joining peer gets a decodable stream.
3. **Gateway composition + end-to-end browser proof** — compose `StrOmSignaler` +
   `StrOmTransport` for `SFU_MODE=sovereign`; prove two real peers exchange decoded media.
4. **Flip `SFU_MODE=sovereign`** — only once G1–G3 prove media flows end-to-end; cover its
   boundary (per-event RLS, tenant isolation, JWT on the media path) in SECURITY §1–§5 first.
5. **Live proofs (carried)** — LiveKit cross-node; admin-ui OIDC once ADR-004 + an IdP land.

Do not advertise any of these as shipped until each functions end-to-end or is re-affirmed
deferred — the discipline that carried phases 16–21.
