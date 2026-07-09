# Goals — phase-29-sovereign-sfu-shared-socket-and-stun

> Seeded from: phase-28 (reflection `summaryForNext`). Phase-28 fixed the `0.0.0.0` ICE candidate-IP
> bug (the gateway now advertises a valid `127.0.0.1:40000` host candidate) and the decode run finally
> reached the media exchange — but produced **no decoded frame**, surfacing two precise str0m-SFU
> blockers. See `docs/PHASE-28-DECODE-RESULT.md` + `docs/PHASE-28-SIGNOFF.md`.

This phase clears those two blockers **in order** (B2 first — it blocks everything), then re-runs the
decode proof and, **only** on a genuine `getStats().framesDecoded > 0`, flips `SFU_MODE=sovereign`.

**Discipline (carried 16→28):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.

---

## G1 — B2: one shared demuxing UDP socket (blocks everything; do first)

Phase-28 evidence: the transport binds `MEDIA_BIND_ADDR:MEDIA_UDP_PORT` (`0.0.0.0:40000`)
**per session**, so a second negotiate fails `udp bind: Address already in use (os error 98)` and its
trickle candidates hit "unknown session" — a single fixed port is a single-session design.

- **G1.1** Replace the per-session bind in `StrOmTransport` with **one shared UDP socket** owned by
  the transport, bound once at `MEDIA_BIND_ADDR:MEDIA_UDP_PORT`.
- **G1.2** **Demux inbound datagrams to the owning `Rtc`** by remote 5-tuple / ICE ufrag (str0m's own
  `chat`/`http-post` examples do exactly this — one socket, `rtc.accepts(&input)` routing). No
  per-session socket, no re-bind.
- **G1.3** ADR the transport change (touches ADR-005 MediaTransport / ADR-006 fan-out socket
  ownership). Keep `MediaConfig` / `RoomRouter` / bridge contracts intact. File-size ≤500; no library
  `unwrap`/`expect`; `tracing` spans across the port boundary.

**Exit:** two sessions negotiate on the one shared port with no `EADDRINUSE`; inbound datagrams route
to the correct `Rtc`; the in-process multi-session test proves the demux (no browser needed).

## G2 — B1: a routable browser candidate (STUN srflx or mDNS resolution)

Phase-28 evidence: Chrome emits its **host** candidates as mDNS `<uuid>.local` hostnames, which str0m
rejects (`ICE bad candidate … invalid IP address syntax`) — so every browser host candidate is
dropped and `remoteCandidates=0`.

- **G2.1** Give the browser a candidate the gateway accepts. Preferred: a **STUN server-reflexive
  path** — configure a STUN server so Chrome gathers an `srflx` candidate (a real IP), which str0m
  accepts. Alternative: resolve `.local` mDNS at the gateway before `add_remote_candidate`.
- **G2.2** Wire the chosen path into the harness/compose (STUN URL in the `RTCPeerConnection` config,
  or the gateway-side resolver) without weakening the auth/authz boundary. ADR if it changes the
  signaling contract.

**Exit:** the gateway accepts ≥1 browser candidate (`remoteCandidates>0`); ICE advances past `new`.

## G3 — Observe a real decoded frame + flip

- **G3.1** Re-run `scripts/run-media-decode.sh`; observe `getStats().framesDecoded > 0`. Capture
  `docs/PHASE-29-DECODE-RESULT.md` (browser assertion + gateway str0m logs).
- **G3.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-29-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G3.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G4 — Carried live proofs (from phase-19…28)

- **G4.1** LiveKit cross-node inbound (`realtime` feature). **G4.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**).
- `SFU_MODE=sovereign` is enabled **only** if G3 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001/003/004/005/006/007 without a new finding (a shared-socket ADR is a refinement,
  not a re-open).
- TURN relay, simulcast, or net-new media features beyond what the decode proof + flip require.

## Starting point (from phase-28)

- Valid gateway host candidate (`127.0.0.1:40000`, p28-c002). `MediaConfig.resolve_advertised_ip`.
- Decode harness + runner (self-mint RS256 JWKS, Keto `view` seed, compose overrides).
- Blockers to clear: **B2** shared demuxing socket (G1), **B1** STUN srflx / mDNS (G2).
