# p27-c002-trickle-ice-over-ws

## Why

The decode times out because ICE never completes (FIND-1/2): the harness never exchanges ICE
candidates, and the gateway's own trickle candidates + connection-state (from
`StrOmTransport::local_signals`) are never relayed to the browser over `/ws/v1/signal` (the route
streams only the `MediaSignaler`). The gateway is the answerer — without the browser's candidate it
can't complete ICE. Wire bidirectional trickle over WS (operator decision).

## What Changes

- **Gateway** (`crates/frf-gateway/src/routes/signal.rs` + `media_bridge`): for a `SFU_MODE=sovereign`
  session, after the offer creates the session, subscribe to `MediaTransport::local_signals(session)`
  and stream its trickle candidates + `ConnectionState` envelopes out over the WS as `ice-candidate`
  frames (alongside the existing `MediaSignaler` stream). Inbound `ice-candidate` frames are already
  routed to the bridge → `add_remote_candidate`.
- **Harness** (`admin-ui/e2e/support/{webrtc-client,decode-probe}.ts`): add `pc.onicecandidate` →
  send `ice-candidate` frames up the WS; on inbound `ice-candidate` → `pc.addIceCandidate`
  (increment `remoteCandidates`).

## Impact

- `crates/frf-gateway/src/routes/signal.rs`, `crates/frf-gateway/src/media_bridge.rs` (expose the
  transport's `local_signals` for a session), harness helpers. Rust tests for the relay.
- No gate flip. Enables ICE to complete → the c003 RoomJoin fan-out → the c004 decode.
