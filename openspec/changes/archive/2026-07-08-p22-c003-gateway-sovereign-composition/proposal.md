# p22-c003 — gateway sovereign composition (media plane wired, gate stays off)

## Why

`StrOmTransport` (the sovereign media engine) is proven at the unit/integration layer, but the
gateway never drives it: `SFU_MODE=sovereign` (`main.rs:269`) builds the **signaling-only**
`StrOmSignaler` and logs "no media will flow", and `signal_service.rs` drives `MediaSignaler`
only. This change composes the sovereign gateway so the media port is **driven from the signal
path** — the last wiring before an end-to-end browser proof (G3.2, carried) could flip the gate.

## Design

A pure, testable **`MediaTransportBridge`** (new `media_bridge.rs`) maps a `SignalEnvelope` to
the right sovereign action on `Arc<StrOmTransport>` (the composition point per ADR-005 — the
gateway may name the concrete engine):

- `SignalKind::Offer` → `create_session(session, tenant, offer_sdp)` → return an `Answer` env.
- `SignalKind::RoomJoin` → `join_room(session, tenant, room)`.
- `SignalKind::IceCandidate` → `add_remote_candidate(session, env)`.
- `SignalKind::RoomLeave` / `Hangup` → `remove_session(session, tenant)`.

The bridge returns any outbound envelope (the answer; local candidates flow via
`local_signals`). The signal service calls the bridge when a transport is configured; the
existing `MediaSignaler` signaling relay is unchanged.

## What Changes

1. **`media_bridge.rs` (new):** `MediaTransportBridge` over `Arc<StrOmTransport>` +
   `handle(env) -> Option<SignalEnvelope>` mapping the kinds above; unit-tested (Offer→answer;
   RoomJoin→join; RoomLeave→remove) with the real `StrOmTransport`.
2. **`main.rs`:** for `SFU_MODE=sovereign`, build **both** `StrOmSignaler` (signaling) and
   `StrOmTransport` (media); hold the transport in app state. Keep the warning that end-to-end
   media is unproven; **do not flip the gate** (hosted remains the production media path).
3. **`signal_service.rs`:** when a `MediaTransportBridge` is present, route inbound envelopes
   through it (relay the answer outbound); signaling relay unchanged.

## Non-goals (carried)

- The end-to-end browser proof (G3.2) + flipping `SFU_MODE=sovereign` (G4) — carried to a
  follow-on. The gate STAYS OFF.
- Renegotiation; per-event Keto RLS on the media path (the media rides the JWT-authed signal
  channel; the per-event boundary is a follow-on).

## Impact

- Affected: `crates/frf-gateway/src/{media_bridge.rs,main.rs,signal_service.rs,lib.rs}`.
- The gateway composes signaling + media for sovereign mode and drives the port from the signal
  path; the bridge is unit-tested. `SFU_MODE=sovereign` stays gated off (media present,
  unproven end-to-end). No lib `unwrap`/`expect` in the bridge; files ≤500.
