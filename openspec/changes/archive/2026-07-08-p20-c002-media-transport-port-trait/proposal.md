# p20-c002 — define the MediaTransport port trait

## Why

ADR-005 (p20-c001) decided the sovereign SFU media plane gets a **new `MediaTransport`
port**, separate from the signaling-only `MediaSignaler`. This change defines that trait in
`frf-ports` — the seam the str0m async engine (c003–c005) implements. No implementation
lands here: `frf-ports` holds no implementations (the absolute dependency rule).

## What Changes

1. **`crates/frf-ports/src/media_transport.rs` (new):** the `MediaTransport` trait — a
   per-session media engine mirroring the `MediaSignaler` + `DynMediaSignaler` shape:
   - `create_session(session_id, tenant_id, offer_sdp) -> answer_sdp` — negotiate + start
     the per-session engine.
   - `add_remote_candidate(session_id, candidate)` — feed an inbound trickle-ICE candidate;
     candidates are carried as `SignalEnvelope` (`SignalKind::IceCandidate`), consistent with
     how signaling already represents them.
   - `local_signals(session_id) -> SignalStream` — outbound stream of the engine's own local
     ICE candidates + connection-state changes, as `SignalEnvelope`s.
   - `connection_state(session_id) -> ConnectionState` — where `ConnectionState` is a
     `#[non_exhaustive]` enum (`New` / `Connecting` / `Connected` / `Disconnected` /
     `Failed`).
   - `remove_session(session_id, tenant_id)` — teardown.
   - **RTP forwarding is NOT on this port** — phase-21.
   - A `DynMediaTransport(Arc<dyn MediaTransport>)` wrapper for runtime `SFU_MODE` selection,
     mirroring `DynMediaSignaler`.
2. **`lib.rs`:** `pub mod media_transport;` + re-exports.

## Non-goals

- Any implementation (c003 implements it in `frf-media-str0m`).
- RTP forwarding methods (phase-21).

## Impact

- Affected: `crates/frf-ports/src/media_transport.rs` (new), `crates/frf-ports/src/lib.rs`.
- The `MediaTransport` seam exists with no adapter dependency; c003 implements the transport
  slice. `#[non_exhaustive]` enums, newtype IDs (from `frf-domain`), `PortError` for errors.
