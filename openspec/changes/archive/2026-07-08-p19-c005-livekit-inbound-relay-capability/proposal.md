# p19-c005 — LiveKit cross-node inbound relay capability

## Why

`LiveKitSignaling::subscribe_signals` is **in-process only**: it serves each session from a
local broadcast channel fed by *this* process's own outbound signals. It does not subscribe
to the LiveKit server's data channel, so a signal published by another gateway node reaches
LiveKit room participants but is **not** re-surfaced through this adapter on this node
(documented at `adapter.rs:26-38`). Phase-19 G2 asks to close this — true cross-node inbound
relay.

## Design decision (operator-approved)

The LiveKit **realtime** Rust SDK pulls a heavy native `libwebrtc` (`webrtc-sys`, hundreds
of MB, platform-specific prebuilt) into the gateway build, and cross-node relay can only be
**proven** against a live LiveKit server. So rather than force libwebrtc into every gateway
build, this change builds the capability behind a **trait seam**:

- A `LiveKitDataSource` port abstracts "yield the next inbound data-channel payload."
- A `spawn_inbound_relay` listen loop pumps a data source → deserialize → forward into the
  per-session broadcast channels (the exact fan-out `send_signal` already does). This
  **plumbing is fully unit-testable** with a mock/in-memory source — no live server, no
  libwebrtc.
- The real libwebrtc-backed data source is scoped to an **off-by-default `realtime` cargo
  feature**, integration-gated against a live LiveKit server. The default gateway build
  stays light.

This is the honest "capability built + integration-gated" exit bar: the relay logic exists,
is wired, and is unit-tested; the live cross-node proof is an integration test behind the
feature.

## What Changes

1. **`inbound.rs` (new):** `LiveKitDataSource` trait + `spawn_inbound_relay` loop
   (deserialize payload → `SignalEnvelope` → forward to session channels; skip malformed
   payloads with a warn, never panic).
2. **`adapter.rs`:** extract the session fan-out into a reusable `fan_out(&SignalEnvelope)`
   method; call it from both `send_signal` and the inbound relay. Add
   `LiveKitSignaling::start_inbound_relay(source)` to spawn the loop.
3. **`Cargo.toml`:** add the `realtime` feature (off by default) as the seam for the future
   libwebrtc-backed source; no heavy dep added in this change (the real source lands with
   the feature when a live target exists).
4. **Docs:** update the `adapter.rs` limitation note to "relay capability present; live
   cross-node proof integration-gated behind `realtime`."

## Non-goals

- Adding the libwebrtc-backed `livekit` realtime crate to the default build (deferred to
  the `realtime` feature + a live target, to keep the gateway build light).
- Proving cross-node relay in CI without a live LiveKit server (integration-gated).

## Impact

- Affected: `crates/frf-media-livekit/src/{adapter.rs,inbound.rs,lib.rs}`, `Cargo.toml`.
- The inbound-relay plumbing exists and is unit-tested; enabling live cross-node relay is a
  feature-gated data source against a live server — no longer an architectural gap, a wiring
  step.
