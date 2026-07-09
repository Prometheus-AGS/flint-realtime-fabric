# Assessment — phase-20-sovereign-sfu-media-loop

> Stage: Assess · 2026-07-08 · Backend: OpenSpec
> Grounds the seeded goals (`goals.md`) against the real str0m/gateway code. This is the
> **XL sovereign SFU media build** — the phase where the point is media actually flowing, so
> the assessment's job is to find the load-bearing design decisions and the browser-interop
> gates *before* planning, not to assume the seed order is codeable as-is.

## Current state (code-grounded)

- **`transport_spike.rs::TransportLoop` (240 L, proven p19-c006):** a single `Rtc` + a
  **blocking** `std::net::UdpSocket`, with `bind()` / `poll_once()` / `feed_datagram()` /
  `drive_timeout()`. This is the proven loop mechanic — but it is a *spike shape* (blocking
  socket, one session, driven by hand in a test), **not** an async per-session task.
- **`sfu.rs::StrOmSignaler` (320 L):** a **signaling-only** relay — `DashMap<(Tenant,Session),
  mpsc::Sender>` + a `rooms` registry. **No `Rtc`, no UDP, no RTP.** Correct for signaling
  (p18-c001 routing fix) but it is *not* the media plane.
- **Gateway `build_media_signaler` (`main.rs:265`):** the `SfuMode::Sovereign` branch
  **warns "no media will flow"** and returns `StrOmSignaler`. `SFU_MODE` defaults to
  `hosted`. The gate is off and honest.
- **`MediaSignaler` port (`frf-ports/src/media.rs:19`):** `send_signal` /
  `subscribe_signals` / `remove_session` — a **signaling interface only. It has no media
  surface** (no `Rtc` lifecycle, no RTP write/forward). A working SFU does not fit this port.
- **`frf-media-str0m` tokio features:** `["rt-multi-thread", "sync"]` — **no `net`**, which
  async `UdpSocket` per-session tasks (G1.1) will require.

## Goal-by-goal grounding

### G1 — Per-session async transport tasks · **LARGE; needs a port/architecture decision** ⚠️

- The c006 `TransportLoop` proves the mechanic but uses a **blocking** socket. G1.1 needs a
  **tokio `UdpSocket`** (`recv_from`.await) + a timer driven by the `poll_output` `Timeout`
  deadline — a real async task, not the spike's hand-driven loop. Requires adding tokio
  `net` (and likely `time`, `macros`) features to the str0m crate.
- G1.2 says "replace the channel-only `StrOmSignaler`." **Finding:** you cannot simply swap
  it — `StrOmSignaler` implements the **signaling** port, while a media SFU is a *different
  concern* (per-session `Rtc` + socket + RTP). **The load-bearing decision:** does the SFU
  media plane (a) extend/replace the `MediaSignaler` port with a media surface, (b) become a
  **new port** (e.g. `MediaTransport`) composed alongside signaling, or (c) live as an
  internal engine the gateway drives directly? The seed under-specified this; it must be an
  **ADR before any G1 code** (mirrors the ADR-004 discipline).
- **Verdict:** codeable but **gated on the port/architecture ADR**. The first change should
  be that decision + a per-session async loop *test* (extend the c006 proof to the
  async/multi-session shape) — not a big-bang rewrite.

### G2 — Trickle ICE · **codeable plumbing, but real ICE is browser-gated** ◐

- G2.1 (pipe `IceCandidate` envelopes ↔ `add_remote_candidate` / emit local) is real,
  testable plumbing over the signaling channel. G2.2 (host/**srflx via STUN** gathering +
  connectivity checks) can only be *proven* against a real peer/STUN server.
- **Verdict:** split — the envelope↔candidate wiring is unit-testable now; the connectivity
  proof is **integration/browser-gated** (honest exit, like p19-c005).

### G3 — DTLS/SRTP + RTP forwarding · **XL; browser-gated proof** ◐

- str0m handles DTLS/SRTP internally, but reaching `Connected` + forwarding
  `Event::MediaData` between peers with per-room fan-out + PLI is the **largest** single
  piece and only *proves out* end-to-end against real browser peers.
- **Verdict:** the forwarding **topology + `writer(mid).write` wiring** is codeable and
  partly unit-testable (fan-out routing), but "two peers exchange media" is a
  **browser-gated integration** exit. This goal alone may warrant splitting.

### G4 — Enable the sovereign gate · **decision/guardrail, downstream of G1–G3** ✅(gated)

- Flipping `SFU_MODE=sovereign` from "warns, no media" to live is a small change **once
  G1–G3 move media** — and must be preceded by covering its boundary in `docs/SECURITY.md`
  §1–§5. Must **not** be flipped early (the whole phase-16–19 discipline).
- **Verdict:** a guardrail change, last; stays gated off until media flows end-to-end.

### G5 — Live cross-node proofs (carry-forward) · **external-infra gated** ◐

- G5.1 LiveKit: implement the libwebrtc-backed `LiveKitDataSource` behind the `realtime`
  feature (p19-c005 seam) — needs the heavy `livekit` realtime crate + a live server.
- G5.2 admin-ui OIDC: **strictly downstream of ADR-004 acceptance + an IdP being stood up**
  (p19-c004). Not codeable until that decision lands.
- **Verdict:** both external-infra gated; likely re-affirmed/integration-gated unless the
  operator stands up the infra this phase.

## Gap summary & phase-shape recommendation

| Goal | Verdict | Codeable this phase? |
|------|---------|----------------------|
| **G1** per-session async tasks | large; needs port/arch ADR first | ◐ ADR + async-loop test, then build |
| **G2** trickle ICE | plumbing codeable; connectivity browser-gated | ◐ wire + integration-gate |
| **G3** DTLS/SRTP + RTP fan-out | XL; end-to-end browser-gated | ◐ topology codeable; proof gated (may split) |
| **G4** enable sovereign gate | small guardrail, last | ✅ gated, after G1–G3 |
| **G5.1** LiveKit live proof | heavy dep + live server | ◐ integration-gated |
| **G5.2** admin-ui OIDC | blocked on ADR-004 + IdP | ⚠️ blocked unless infra stood up |

**Recommended phase shape (for `/kbd-plan`):**
1. **ADR first** — the SFU media-plane architecture / port decision (G1's blocker). Nothing
   in G1–G3 should be coded before this, exactly as ADR-004 gated the OIDC work.
2. **Async transport loop** — evolve `TransportLoop` into a per-session tokio task (add
   `net`/`time` features), proven by an async loop test.
3. **Trickle ICE wiring** — envelope↔candidate plumbing (unit-testable) + integration-gated
   connectivity.
4. **RTP forwarding topology** — the fan-out + `writer` wiring; end-to-end proof
   browser-gated. **Consider splitting G3** into its own sub-phase if it dwarfs the rest.
5. **Enable the gate (G4)** only if media flows; else re-affirm gated-off.
6. **Carry-forwards (G5)** — integration-gated / blocked-on-infra; do them only if the
   operator provides the live server / IdP this phase.

This keeps the phase-16–19 discipline: an ADR for the load-bearing unknown, real capability
built + unit-tested, browser/infra proofs honestly integration-gated, and
`SFU_MODE=sovereign` off until media actually flows.

## Open questions for `/kbd-plan`

1. **SFU media-plane architecture (the big one):** extend `MediaSignaler`, add a new
   `MediaTransport` port, or an internal gateway-driven engine? (Assessment: ADR first,
   recommend a new port so signaling stays one concern.)
2. **G3 split:** should RTP forwarding + DTLS be its own sub-phase (phase-21) given its
   size, with phase-20 stopping at a connected-but-no-media-forwarded milestone?
3. **G5 infra:** will the operator stand up a live LiveKit server and/or an IdP this phase,
   or are G5.1/G5.2 integration-gated / carried again?
4. **Exit realism:** confirm that "two peers exchange media" (G3) and ICE connectivity (G2)
   are **browser-gated integration** exits, not unit-test exits (matches p19's honest bar).
