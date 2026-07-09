# p20-c003 — str0m async per-session media loop

## Why

ADR-005 + the `MediaTransport` port (c002) define the seam; the c006 `TransportLoop` proved
the sans-I/O mechanics but with a **blocking** socket, one session, driven by hand in a test.
This change builds the real engine: an **async per-session task** (tokio `UdpSocket`) that
drives a str0m `Rtc` and implements the transport slice of `MediaTransport`, so the gateway
has a sovereign media engine that binds real sockets and turns the loop under tokio.

## Design

- **`StrOmTransport`** implements `MediaTransport`; it owns a `DashMap<SessionId,
  SessionHandle>`. Each `SessionHandle` carries a command `mpsc::Sender`, a
  `watch::Receiver<ConnectionState>`, and a `broadcast::Sender<SignalEnvelope>` for outbound
  local signals.
- **`create_session`** negotiates the offer→answer (reusing the proven negotiation), binds a
  tokio `UdpSocket`, adds the socket's real addr as a host candidate, spawns the async
  driver, registers the handle, and returns the answer SDP.
- **The async driver (`run_session`)** is the async translation of str0m's canonical loop
  (examples/http-post.rs): drain `poll_output()` until `Timeout` (on `Transmit` → `send_to`;
  on `Event` → update state / surface); then `tokio::select!` between
  `tokio::time::sleep_until(deadline)` (→ `Input::Timeout`) and `socket.recv_from`
  (→ `Input::Receive`); `handle_input`; loop. On `Event::Connected` /
  `IceConnectionStateChange`, publish the new `ConnectionState` to the `watch`.
- **No RTP forwarding** — the driver reaches connection lifecycle only (phase-21 adds media).

## What Changes

1. **`Cargo.toml`:** tokio `net`, `time`, `macros` features (async socket + timer + select).
2. **`session.rs` (new):** `StrOmTransport` (`MediaTransport` impl) + `SessionHandle` +
   `run_session` async driver + `SessionCommand`. Keep ≤500 L (split an `ice`/`driver`
   sub-file if it grows — c004 may take the ICE half).
3. **`lib.rs`:** re-export `StrOmTransport`.
4. **Test:** `create_session` on a real tokio socket negotiates + spawns the driver, and the
   loop turns (the session's `ConnectionState` moves off `New`, or an outbound transmit is
   observed) — the async extension of the c006 proof. `remove_session` tears the task down.

## Non-goals

- RTP `MediaData` forwarding, per-room fan-out, PLI (phase-21).
- Enabling `SFU_MODE=sovereign` (still gated off — the engine reaches connected, not media).
- Full ICE trickle wiring over the signaling path (c004).

## Impact

- Affected: `crates/frf-media-str0m/src/{session.rs,lib.rs}`, `Cargo.toml`.
- A per-session async str0m engine implementing `MediaTransport` exists and turns over a real
  tokio socket, proven by a test. No library `unwrap`/`expect`.
