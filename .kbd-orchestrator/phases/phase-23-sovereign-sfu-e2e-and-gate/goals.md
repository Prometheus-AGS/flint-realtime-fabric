# Goals — phase-23-sovereign-sfu-e2e-and-gate

> Seeded from: phase-22 (p22-c004). Built on the composed sovereign media plane
> (`StrOmTransport` + `MediaTransportBridge` + `RoomRouter`) + ADR-005/006.
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-reflect`.

Phase-22 composed and layer-proved the sovereign media plane end to end **in code**: N-peer
fan-out, PLI/keyframe forwarding, and the gateway driving `MediaTransport` from the signal
path. What remains is the **external proof** — a real browser peer exchanging *decoded* media
through the sovereign SFU — and, only once that passes, **flipping `SFU_MODE=sovereign` on**.
This is the phase that makes sovereign media a shippable path.

**Discipline (carried 16–22):** `SFU_MODE=sovereign` flips on **only** once media provably
flows end-to-end to a real browser peer; else it stays gated off. No "healthy but does
nothing." Update `docs/SECURITY.md` §6 + the CHANGELOG as each lands; re-run the gate suite.

---

## G1 — Browser E2E test harness

- **G1.1** A WebRTC test page (a minimal offerer/answerer or a browser joining a room via the
  gateway's signal channel) + a **Playwright** harness that drives it against a running
  `SFU_MODE=sovereign` gateway.
- **G1.2** CI wiring: run the harness in Dagger (Node + Chromium), gated behind a flag so it
  only runs where a browser is available.

**Exit:** the harness connects a real browser peer to the sovereign gateway and reaches
`Connected`, proven in CI (or documented as locally-run if CI browser infra is unavailable).

## G2 — Decoded-media end-to-end proof

- **G2.1** Two real peers (browser ↔ browser, or browser ↔ a headless str0m peer) exchange
  media through the sovereign SFU, and the receiver **decodes** a frame (the proof phase-21/22
  honestly gated). Exercises real ICE + DTLS/SRTP + RTP forwarding + PLI end to end.

**Exit:** a receiver decodes media relayed by the sovereign SFU, proven by the harness.

## G3 — Security boundary for the media path (before the flip)

- **G3.1** Cover the sovereign media path in `docs/SECURITY.md` §1–§5: JWT verification on the
  signal/media channel (already authed), **per-event Keto RLS** (`check(subject, "view",
  object)`) on media fan-out where required, and **tenant isolation** (the `RoomRouter` keys
  rooms by `(TenantId, room)` — verify no cross-tenant fan-out).

**Exit:** the media path's authz/isolation boundary is documented + enforced.

## G4 — Flip `SFU_MODE=sovereign`

- **G4.1** Flip the gateway sovereign branch from the "unproven end-to-end" warning to a live
  production path — **only** after G2 proves decoded media flows and G3 covers the boundary.

**Exit:** `SFU_MODE=sovereign` moves real media in production, or stays gated with rationale.

## G5 — Live cross-node / OIDC proofs (carried from phase-19/20/21/22)

- **G5.1** LiveKit cross-node inbound: the libwebrtc-backed `LiveKitDataSource` behind the
  `realtime` feature; prove cross-node relay vs a live server.
- **G5.2** admin-ui OIDC authorization-code flow — once ADR-004 is Accepted + an IdP exists.

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  `docs/SECURITY.md` §6 + CHANGELOG.
- Release gate suite green; each code change passes the QA gate (**verdict read before archive**).
- `SFU_MODE=sovereign` is enabled **only** if decoded media provably flows end-to-end.

## Non-goals

- Re-opening settled decisions (ADR-001/003/004/005/006) without a new finding.
- Net-new media features beyond what the E2E proof + gate flip require.

## Starting point (proven in phase-22)

- `crates/frf-media-str0m/src/{room.rs,driver.rs,session.rs}` — N-peer fan-out + PLI forwarding
  + the async engine, layer-proven.
- `crates/frf-gateway/src/media_bridge.rs` — the signal→MediaTransport bridge.
- `crates/frf-gateway/src/{main.rs,signal_service.rs}` — the sovereign composition (gate off).
- ADR-005 (MediaTransport port), ADR-006 (RTP fan-out).
