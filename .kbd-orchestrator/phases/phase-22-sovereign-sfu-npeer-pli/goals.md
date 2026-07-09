# Goals — phase-22-sovereign-sfu-npeer-pli

> Seeded from: phase-21 (p21-c005), built on `crates/frf-media-str0m/src/{session,driver,room}.rs`
> (the async engine + `RoomRouter` 1-to-1 forwarding) + ADR-005/006.
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-reflect`.

Phase-21 wired and layer-proved **1-to-1** RTP forwarding and proved two peers reach
DTLS-connected in-process. What remains before `SFU_MODE=sovereign` can flip on: **N-peer
per-room fan-out + PLI**, and the **end-to-end browser media proof** that the SFU actually
relays decoded media between real peers. This phase completes the sovereign media plane to a
shippable state.

**Discipline (carried 16–21):** `SFU_MODE=sovereign` flips on **only** once media flows
end-to-end to real peers; else it stays gated off. No "healthy but does nothing." Update
`docs/SECURITY.md` §6 + the CHANGELOG as each lands; re-run the gate suite at close.

---

## G1 — N-peer per-room fan-out

The `RoomRouter` (c004) already fans a frame to all *other* room members; phase-21 only
proved the 2-peer case.

- **G1.1** Prove N-peer fan-out: 3+ sessions in a room, one sends, all others receive
  (in-process where feasible; the router logic is general, the proof is the gap).
- **G1.2** Fan-out topology decisions: SFU (relay each sender to all) vs selective forwarding;
  simulcast/RID handling if needed (str0m surfaces `rid` on `MediaData`).

**Exit:** N peers in a room each receive the others' media, proven or integration-gated.

## G2 — PLI / keyframe handling + renegotiation

- **G2.1** When a peer joins mid-stream, request a keyframe (PLI) from the senders so the new
  peer gets a decodable stream; forward str0m's PLI/keyframe-request events.
- **G2.2** Renegotiation as room membership / tracks change.

**Exit:** a late-joining peer receives a decodable stream (PLI honored), browser-gated.

## G3 — End-to-end browser media proof + gateway composition

- **G3.1** Compose the gateway for `SFU_MODE=sovereign`: `StrOmSignaler` (signaling) +
  `StrOmTransport` (media) side by side (ADR-005), wiring `create_session`/`join_room`/
  `add_remote_candidate`/`local_signals` to the signaling channel.
- **G3.2** Prove end-to-end: a real browser peer connects, and media relays through the
  sovereign SFU to another peer (the proof phase-21 honestly gated).

**Exit:** two real peers exchange decoded media through the sovereign gateway path.

## G4 — Flip `SFU_MODE=sovereign` (only when media flows)

- **G4.1** Flip the gate from "warns, no media" to the live composed path **only** after
  G1–G3 prove media flows end-to-end. Cover its boundary in `docs/SECURITY.md` §1–§5 first
  (per-event RLS, tenant isolation, JWT verification on the media path).

**Exit:** `SFU_MODE=sovereign` moves real media in production, or stays gated with rationale.

## G5 — Live cross-node / OIDC proofs (carried from phase-19/20/21)

- **G5.1** LiveKit cross-node inbound: the libwebrtc-backed `LiveKitDataSource` behind the
  `realtime` feature; prove cross-node relay vs a live server.
- **G5.2** admin-ui OIDC authorization-code flow — once ADR-004 is Accepted + an IdP exists.

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  `docs/SECURITY.md` §6 + CHANGELOG.
- Release gate suite green; each code change passes the QA gate.
- `SFU_MODE=sovereign` is enabled **only** if media flows end-to-end; else it stays gated off.

## Non-goals

- Re-opening settled decisions (ADR-001/003/004/005/006) without a new finding.
- Net-new features beyond N-peer fan-out, PLI, the end-to-end proof, and the carried live proofs.

## Starting point (proven in phase-21)

- `crates/frf-media-str0m/src/room.rs` — `RoomRouter` (fan-out + bounded forwarding channels).
- `crates/frf-media-str0m/src/driver.rs` — the forward arm + `Event::MediaData` routing.
- `crates/frf-media-str0m/src/session.rs` — `StrOmTransport` + `join_room`; two-peer connected
  proof (`two_peers_reach_dtls_connected`) + the 1-to-1 forwarding wiring test.
- ADR-005 (MediaTransport port), ADR-006 (RTP fan-out).
