# Assessment — phase-21-sovereign-rtp-forwarding

> Stage: Assess · 2026-07-08 · Backend: OpenSpec
> Grounds the seeded goals (`goals.md`) against the real str0m 0.21 media API + the phase-20
> engine. This is the phase where media actually flows — the assess job is to find the
> load-bearing design (cross-task RTP fan-out) and the browser-gated proofs before planning.

## Current state (code-grounded)

- **`session.rs` is 498 / 500 lines.** Confirmed. It **must be split before** any RTP code,
  or the first engine edit breaches the file-size limit. This is the first change.
- **`StrOmTransport` (phase-20):** each session runs an **isolated** async driver task that
  owns its own `Rtc` + tokio `UdpSocket` (`run_session`, spawned per `create_session`). There
  is **no cross-session channel** today — drivers do not talk to each other.
- **`MediaTransport` port (`frf-ports`):** answerer-only (`create_session(offer)→answer`), no
  RTP method (ADR-005 deferred it here). No offerer path.
- **The `#[ignore]`d `two_peers_reach_dtls_connected`** test marks the offerer/peer gap.

## str0m 0.21 media API (grounded)

- **RTP write:** `rtc.writer(mid: Mid) -> Option<Writer>`; `Writer::write(pt: Pt, wallclock:
  Instant, rtp_time: MediaTime, data: impl Into<Arc<[u8]>>) -> Result<(), RtcError>`.
- **Inbound media:** `Event::MediaData(MediaData { mid, pt, rid, data, .. })`.
- **Forwarding = read `Event::MediaData` on session A's driver → write it to every *other*
  session's `Rtc` in the room via `writer(mid).write(pt, .., data)`.**
- **Offerer:** `sdp_api().add_media(..).apply() -> (offer, pending)` then
  `accept_answer(pending, answer)` — feasible (str0m's own tests use it).
- `Rtc::is_connected()` tells us when a session may write.

## Goal-by-goal grounding

### G0 (implied, first) — split `session.rs` · **mandatory, do first** ⚠️

- 498/500 lines. Any RTP addition breaches immediately. Extract the driver loop (`run_session`
  + helpers) into a `driver.rs`, leaving `session.rs` = the `StrOmTransport`/`MediaTransport`
  surface. **This must be change #1** — a pure refactor, behavior-preserving, tests unchanged.

### G1 — Offerer/peer role + two-peer connected proof · **codeable; unblocks G2's proof** ◐

- `sdp_api().add_media().apply()` gives the offerer path. Adding it (on the port or a
  test/peer harness) lets two `StrOmTransport` sessions negotiate + drive ICE/DTLS over
  loopback to `Connected` — un-`#[ignore]`ing `two_peers_reach_dtls_connected`.
- **Risk:** loopback ICE connectivity timing can be flaky in CI; the exit may be "proven
  locally, `#[ignore]` in CI" — honest, matching phase-20's bar.
- **Verdict:** codeable; it is the prerequisite for *any* in-process RTP proof.

### G2 — RTP forwarding · **XL; the crux is CROSS-TASK fan-out, not the write call** ⚠️

- The write itself (`writer(mid).write`) is a few lines. The **load-bearing problem** is
  architectural: each session's `Rtc` is in an **isolated driver task**, so forwarding A→B is
  **cross-task message passing** — A's driver must hand its `MediaData` to B's driver, which
  owns B's `Rtc`. This needs (a) a **room registry** (session → co-room peers), and (b) a
  **per-session forwarding channel** (an `mpsc<ForwardedMedia>` the driver `select!`s on and
  writes). `MediaData`'s payload is `Arc<[u8]>` (cheap to clone across the channel).
- **This is an ADR-worthy design decision** (the forwarding topology / where the room registry
  lives / the `MediaTransport` RTP method shape) — recommend an ADR or a design note before
  the code, consistent with ADR-004/005.
- **Verdict:** codeable but **the cross-task fan-out design must be settled first**; the
  end-to-end "two peers exchange media" proof is **browser/peer-gated** (needs G1's harness or
  a real browser).

### G3 — Per-room fan-out + PLI · **builds on G2; browser-gated proof** ◐

- Per-room fan-out is the registry from G2 generalized to N peers. **PLI/keyframe** handling +
  renegotiation is real WebRTC complexity, provable only against decoding peers.
- **Verdict:** the fan-out topology is codeable + partly unit-testable (routing); PLI + "N
  peers decode" is browser-gated. **May warrant splitting** if it dwarfs G2.

### G4 — Enable the sovereign gate · **small guardrail, last, only if media flows** ✅(gated)

- Flip `SFU_MODE=sovereign` in `build_media_signaler` to compose `StrOmTransport` — but
  `StrOmTransport` implements `MediaTransport`, **not** `MediaSignaler`, so the gateway must
  compose *both* (signaling via `StrOmSignaler` + media via `StrOmTransport`). Cover the
  boundary in SECURITY §1–§5 first. **Only when media actually flows.**
- **Verdict:** guardrail, last; stays gated off until G2–G3 move media.

### G5 — Live proofs (carried) · **external-infra gated** ◐

- G5.1 LiveKit `realtime` data source (heavy libwebrtc + live server); G5.2 admin-ui OIDC
  (blocked on ADR-004 + an IdP). Both external-infra gated — re-affirm unless infra is stood up.

## Gap summary & phase-shape recommendation

| Goal | Verdict | Codeable this phase? |
|------|---------|----------------------|
| **G0** split `session.rs` | mandatory refactor | ✅ **first change** |
| **G1** offerer + connected proof | codeable; unblocks RTP proof | ◐ prove locally / CI-gate |
| **G2** RTP forwarding | XL; cross-task fan-out is the crux (ADR-worthy) | ◐ design first, then build; e2e browser-gated |
| **G3** per-room fan-out + PLI | builds on G2; PLI browser-gated | ◐ topology codeable; may split |
| **G4** enable sovereign gate | small guardrail, last, gated | ✅ only if media flows |
| **G5** live proofs | external-infra gated | ◐ re-affirm unless infra provided |

**Recommended phase shape (for `/kbd-plan`):**
1. **Split `session.rs` → `driver.rs`** (unblocks everything; behavior-preserving).
2. **Offerer role + un-gate the two-peer connected proof** (G1) — the in-process harness RTP
   needs.
3. **ADR / design note for the cross-task RTP fan-out** (room registry + forwarding channels +
   the `MediaTransport` RTP method) — the G2 crux, before code.
4. **RTP forwarding** against that design; **per-room fan-out**; PLI. **Consider splitting G3
   (PLI / N-peer) into phase-22** if it dwarfs G2 — the same milestone discipline as phase-20.
5. **Enable the gate (G4)** only if media flows end-to-end; else re-affirm gated-off.
6. **Live proofs (G5)** — integration-gated / infra-dependent.

Keeps the 16–20 discipline: refactor for headroom, an ADR for the load-bearing unknown, real
capability built + unit-tested, browser/infra proofs honestly gated, `SFU_MODE=sovereign` off
until media actually flows.

## Open questions for `/kbd-plan`

1. **RTP fan-out architecture (the big one):** where does the room registry live (in
   `StrOmTransport`? a new type?), and how do session drivers exchange `MediaData` — an
   `mpsc` per session fed by a central router, or direct peer channels? (Assessment: ADR
   first; recommend a central room registry in `StrOmTransport` + per-session forwarding
   `mpsc`.)
2. **G3 split:** should PLI + N-peer fan-out be its own phase-22, with phase-21 stopping at
   1-to-1 RTP forwarding between two connected peers?
3. **Proof bar:** confirm "two peers exchange media" and PLI are **browser/peer-gated
   integration** exits (in-process loopback where feasible), not unit-test exits — matching
   phases 19–20.
4. **G4 composition:** confirm the gateway composes both `StrOmSignaler` (signaling) and
   `StrOmTransport` (media) for `SFU_MODE=sovereign`, per ADR-005.
