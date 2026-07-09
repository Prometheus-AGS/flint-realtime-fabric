# Goals — phase-20-sovereign-sfu-media-loop

> Seeded from: phase-19 (p19-c007), built on `crates/frf-media-str0m/SPIKE-FINDINGS.md`
> (post-c006 transport-loop proof) and the phase-19 carry-forwards.
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-reflect`.

Phase-19 de-risked the two str0m unknowns (negotiation p18-c006; **live-UDP transport loop
p19-c006**) and shipped the achievable federation/auth/cleanup work. What remains is the
**XL sovereign SFU media build** plus the **live cross-node proofs** that need a real
server/browser. This phase turns the proven transport-loop mechanics into a working
sovereign SFU that actually moves media — the point at which `SFU_MODE=sovereign` may be
enabled.

**Discipline (carried 16–19):** `SFU_MODE=sovereign` stays gated off until media flows
end-to-end to a real peer. No "healthy but does nothing." Update `docs/SECURITY.md` §6 and
the CHANGELOG as each lands; re-run the gate suite at close.

---

## G1 — Per-session async transport tasks (build on the c006 spike)

Promote the c006 `TransportLoop` (single `Rtc` + blocking socket, spike shape) into a
production per-session architecture.

- **G1.1** A per-session async task: tokio `UdpSocket`, `recv_from` → `handle_input(Receive)`,
  a timer driven by the `poll_output()` `Timeout` deadline, `Transmit` → `send_to`.
- **G1.2** Replace the channel-only `StrOmSignaler` model with per-session `Rtc` + socket +
  task, keyed by session; wire it under `SFU_MODE=sovereign` (still gated until G4).

**Exit:** a per-session task drives a real `Rtc` over async UDP; proven by a test that
turns the loop (extending the c006 proof to the async/multi-session shape).

## G2 — Trickle ICE over the signaling channel

- **G2.1** Pipe inbound `IceCandidate` signal envelopes into `rtc.add_remote_candidate`;
  emit the SFU's local candidates back out through `SignalService`.
- **G2.2** Real candidate gathering (host/srflx via STUN) beyond the single host candidate
  the spikes use; connectivity checks.

**Exit:** an ICE connectivity check completes against a real peer (integration/browser-gated).

## G3 — DTLS/SRTP + RTP forwarding

- **G3.1** Drive the DTLS handshake (str0m-internal) to `Connected`; confirm SRTP keying.
- **G3.2** Forward `Event::MediaData` between session peers (`rtc.writer(mid).write`), keyed
  by mid/track, with **per-room fan-out topology** and keyframe (PLI) handling + renegotiation.

**Exit:** two peers exchange real media through the sovereign SFU (browser-gated proof).

## G4 — Enable the sovereign gate (only when media flows)

- **G4.1** Flip `SFU_MODE=sovereign` from "warns, no media" to a live path **only** once
  G1–G3 move media end-to-end. Cover its boundary in `docs/SECURITY.md` §1–§5 first.

**Exit:** `SFU_MODE=sovereign` moves real media, or stays gated with an updated rationale.

## G5 — Live cross-node proofs (carry-forward from phase-19)

- **G5.1** LiveKit cross-node inbound: implement the libwebrtc-backed `LiveKitDataSource`
  behind the `realtime` feature (p19-c005 seam) and prove a signal published on one node
  reaches a subscriber on another via a live LiveKit server.
- **G5.2** (if ADR-004 Accepted + IdP stood up) the admin-ui OIDC authorization-code flow —
  otherwise carry forward.

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale
  in `docs/SECURITY.md` §6 + CHANGELOG.
- Release gate suite green; each code change passes the QA gate.
- `SFU_MODE=sovereign` is enabled **only** if media actually flows; else it stays gated off.

## Non-goals

- Re-opening settled decisions (ADR-001/003/004) without a new finding.
- Net-new features beyond the sovereign SFU media loop and the carry-forward proofs.

## Starting point (proven in phase-19)

- `crates/frf-media-str0m/src/transport_spike.rs` — `TransportLoop` (bind + `poll_once` +
  `feed_datagram`), the sans-I/O loop over a real socket. `SPIKE-FINDINGS.md` scopes the rest.
- `crates/frf-media-livekit/src/inbound.rs` — the `LiveKitDataSource` seam + `realtime` feature.
- ADR-004 — the admin-ui OIDC IdP decision (pending Accept).
