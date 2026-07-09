# ADR-005: Sovereign SFU Media Plane — the `MediaTransport` Port

## Status

Proposed — 2026-07-08 (p20-c001)

Gates all sovereign SFU media-engine work in phase-20 (per-session `Rtc` loop, ICE, DTLS,
and — in phase-21 — RTP forwarding). No engine code lands before this is decided.

## Context

Phase-19 proved the two str0m unknowns: negotiation (p18-c006) and the sans-I/O UDP
transport loop over a real socket (p19-c006, `transport_spike.rs`). Phase-20 turns that into
a real sovereign media plane. Grounding the phase-20 goals against the code surfaced the
load-bearing design question:

- **`MediaSignaler` (`frf-ports/src/media.rs`) is signaling-only** — `send_signal` /
  `subscribe_signals` / `remove_session`, documented "Never stores media — signaling only."
- **`StrOmSignaler` (`frf-media-str0m/src/sfu.rs`) is a signaling relay** — a `DashMap` of
  `mpsc` senders + a rooms registry. **No `Rtc`, no UDP, no RTP.**
- A working SFU media plane is a *different concern*: per-session `Rtc` + a UDP socket + an
  event loop + (phase-21) RTP forwarding. It does not fit the signaling port.

So "replace the channel-only signaler" (the phase-20 seed's wording) is not a swap — the
media plane needs its own home in the ports layer. CLAUDE.md's **one-port-per-adapter** and
the **absolute dependency rule** (adapters implement exactly one port; `frf-ports` holds no
implementations) constrain the answer.

## Decision

### Options

**Option A — a new `MediaTransport` port (recommended).**
Keep `MediaSignaler` for signaling. Add a separate `MediaTransport` trait in `frf-ports` for
the per-session media engine; `frf-media-str0m` implements **both** ports as distinct
concerns, and the gateway composes them side by side (a `DynMediaTransport` wrapper mirrors
the existing `DynMediaSignaler` for runtime selection under `SFU_MODE`).

- **Pros:** signaling and media transport stay separate concerns (each a focused port);
  honors one-port-per-adapter (str0m implements two *distinct* ports, not one overloaded
  one); `frf-ports` stays implementation-free; the hosted (LiveKit) path is unaffected —
  it simply has no `MediaTransport` impl.
- **Cons:** a second port + a `Dyn` wrapper to maintain; the gateway composes two media
  objects for sovereign mode.

**Option B — extend `MediaSignaler` with media-lifecycle methods.**
Add `Rtc`/RTP lifecycle onto the existing port.

- **Cons:** conflates signaling and media transport into one port; every `MediaSignaler`
  impl (incl. LiveKit, which has no sovereign `Rtc`) must stub the media methods; muddies
  the seam the phase-18 routing fix cleaned up. Rejected.

**Option C — an internal gateway-driven engine (no port).**
The gateway drives a str0m media engine directly, bypassing the ports layer for media.

- **Cons:** breaks the absolute dependency rule for the media path (interface reaching into
  a concrete engine); untestable via the port seam; not swappable. Rejected.

### Recommendation

**Option A — a new `MediaTransport` port.** It is the only option that keeps the clean
seam, honors one-port-per-adapter, and leaves the hosted path untouched.

### Proposed `MediaTransport` surface (refined in c002)

A per-session media engine, mirroring the `MediaSignaler` + `DynMediaSignaler` shape:

- `create_session(session_id, tenant_id, offer) -> answer` — negotiate + start the async
  per-session `Rtc` loop (c003).
- `add_remote_candidate(session_id, candidate)` and a way to surface **local** candidates
  outbound (trickle ICE, c004) — carried as `SignalEnvelope`/`IceCandidate` so signaling
  remains the transport for candidates.
- `connection_state(session_id) -> …` / an event surface for reaching DTLS-`Connected` (c005).
- `remove_session(session_id, tenant_id)` — teardown.
- **RTP forwarding is explicitly phase-21** — not on this port yet; phase-20 stops at
  DTLS-connected.

The exact signatures are settled in c002 (the trait). `#[non_exhaustive]` on public enums;
newtype session IDs; `PortError` for errors; no implementation in `frf-ports`.

## Consequences

- **c002** defines the `MediaTransport` trait in `frf-ports` (no impl). **c003–c005**
  implement it in `frf-media-str0m` up to DTLS-connected. **Phase-21** adds RTP forwarding
  (its own port method or a follow-on) + per-room fan-out + PLI.
- `frf-media-str0m` will implement **two** ports (`MediaSignaler` + `MediaTransport`) — this
  is two *distinct* concerns, consistent with one-port-per-adapter (not one port doing two
  jobs).
- `SFU_MODE=sovereign` stays **gated off** until media actually forwards (phase-21) — this
  ADR does not enable it.
- The gateway's `build_media_signaler` gains a sibling that composes the `MediaTransport`
  for sovereign mode (wired when the engine lands, not in this ADR).

## Related

- p19-c006 — the `transport_spike.rs` transport-loop proof this port formalizes.
- ADR-004 — same discipline: an ADR for the load-bearing decision before implementation.
- [ADR-007](adr-007-media-path-authz.md) — media-path authorization (Keto `view` at room-join),
  composed over this port in the gateway bridge; a **G3 precondition for the gate flip**.
- CLAUDE.md — one-port-per-adapter; the absolute dependency rule; `frf-ports` holds no impls.
