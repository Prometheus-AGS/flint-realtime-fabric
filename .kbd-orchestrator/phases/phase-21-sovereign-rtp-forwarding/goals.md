# Goals — phase-21-sovereign-rtp-forwarding

> Seeded from: phase-20 (p20-c006), built on `crates/frf-media-str0m/src/session.rs`
> (`StrOmTransport` async engine, DTLS-connected milestone) + `SPIKE-FINDINGS.md`.
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-reflect`.

Phase-20 built the sovereign SFU up to **DTLS-connected**: a `MediaTransport` port, an async
per-session str0m engine, trickle-ICE wiring, and crypto install. What remains is the actual
**media**: forwarding RTP between peers with per-room fan-out — the point at which
`SFU_MODE=sovereign` may finally be enabled. This phase also unblocks the two-peer /
browser proofs that phase-20 honestly integration-gated.

**Discipline (carried 16–20):** `SFU_MODE=sovereign` stays gated off until media flows
end-to-end to a real peer. No "healthy but does nothing." Update `docs/SECURITY.md` §6 + the
CHANGELOG as each lands; re-run the gate suite at close.

---

## G1 — Offerer / peer role + the two-peer connected proof

The `MediaTransport` port only exposes the answerer path (`create_session(offer)→answer`).
RTP forwarding + the connected proof need a peer in the offerer role.

- **G1.1** Add an offerer-capable path (create an offer, accept an answer) — either on the
  port or a test/peer harness — so two sessions can complete ICE + DTLS in-process.
- **G1.2** Un-`#[ignore]` `two_peers_reach_dtls_connected`: two sessions over loopback reach
  `ConnectionState::Connected` through the real engine, proven by a test.

**Exit:** two in-process sessions reach DTLS-connected, or the proof stays browser-gated with
an updated rationale.

## G2 — RTP forwarding between peers

- **G2.1** In the session driver, handle `Event::MediaData` and forward it to the other
  peers in the room via `rtc.writer(mid).write(..)`, keyed by mid/track.
- **G2.2** Extend `MediaTransport` (or add a media surface) for the forwarding topology — the
  RTP method ADR-005 deferred.

**Exit:** a media packet sent by one peer is delivered to another through the sovereign SFU
(browser/peer-gated proof), or re-affirmed deferred.

## G3 — Per-room fan-out topology + PLI

- **G3.1** A per-room registry mapping sessions → the peers they forward to; fan-out on
  `MediaData`.
- **G3.2** Keyframe (PLI) handling + renegotiation so a newly-joined peer gets a decodable
  stream.

**Exit:** N peers in a room each receive the others' media; PLI requests are honored.

## G4 — Enable the sovereign gate (only when media flows)

- **G4.1** Flip `SFU_MODE=sovereign` in the gateway from "warns, no media" to a live path
  that composes `StrOmTransport` — **only** once G2–G3 move media end-to-end. Cover its
  boundary in `docs/SECURITY.md` §1–§5 first.

**Exit:** `SFU_MODE=sovereign` moves real media, or stays gated with an updated rationale.

## G5 — Live cross-node proofs (carried from phase-19/20)

- **G5.1** LiveKit cross-node inbound: implement the libwebrtc-backed `LiveKitDataSource`
  behind the `realtime` feature (p19-c005 seam); prove cross-node relay vs a live server.
- **G5.2** admin-ui OIDC authorization-code flow — once ADR-004 is Accepted + an IdP is
  stood up (else carry forward again).

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  `docs/SECURITY.md` §6 + CHANGELOG.
- Release gate suite green; each code change passes the QA gate.
- `SFU_MODE=sovereign` is enabled **only** if media actually flows; else it stays gated off.

## Non-goals

- Re-opening settled decisions (ADR-001/003/004/005) without a new finding.
- Net-new features beyond RTP forwarding and the carried live proofs.

## Starting point (proven in phase-20)

- `crates/frf-media-str0m/src/session.rs` — `StrOmTransport` async engine reaching
  DTLS-connected; `wait_for_connected`; the `#[ignore]`d two-peer proof to un-gate.
- `crates/frf-ports/src/media_transport.rs` — the `MediaTransport` port (RTP method pending).
- `crates/frf-media-str0m/src/ice.rs` — trickle-ICE envelope helpers.
- ADR-005 — the media-plane port decision (RTP explicitly deferred here).
