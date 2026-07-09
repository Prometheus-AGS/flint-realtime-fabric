# p19-c006 — str0m live-UDP transport-loop spike

## Why

The phase-18 spike (p18-c006) proved the str0m 0.21 **negotiation** round-trip (offer →
answer) but stopped there. `SPIKE-FINDINGS.md` names the **load-bearing unknown** for a
sovereign SFU: the sans-I/O UDP event loop — bind a real socket, drive
`rtc.handle_input(Input::Receive/Timeout)` + `rtc.poll_output()` →
`Output::{Transmit,Timeout,Event}`, and write outbound datagrams. That transport loop is
where WebRTC interop actually breaks and is hard to test without a browser. This change is
the **thin, honest slice** that de-risks it: prove the loop turns over a real UDP socket,
in a test, without a browser. The full media fan-out (trickle ICE, DTLS/SRTP, RTP
forwarding, per-room topology) remains the split-out **phase-20**.

## What Changes

1. **`transport_spike.rs` (new):** a `TransportLoop` that owns a real bound
   `std::net::UdpSocket` + an `Rtc`, exposing:
   - `bind()` — binds a UDP socket on loopback and adds its **actual local addr** as a host
     `Candidate` (unlike the negotiation spike's port-0 placeholder).
   - `poll_once()` — drains `rtc.poll_output()`: on `Transmit` actually `send_to` the
     datagram over the socket; on `Timeout` returns the deadline; on `Event` surfaces it.
     Returns a `PollStep` enum describing what the loop did.
   - `feed_datagram(buf, from)` — constructs a real `Input::Receive` (proto UDP, the
     socket's addr as destination) and calls `rtc.handle_input`.
2. **Test:** bind a real socket, negotiate a real offer (via the existing spike), then
   drive `poll_once` and assert the loop reaches a `Timeout`/`Transmit` step (the event
   loop turns) — and that feeding a non-STUN datagram is handled without panic. Proves the
   sans-I/O loop mechanics over a real socket; no browser, no full ICE.
3. **`SPIKE-FINDINGS.md`:** record what the transport-loop spike proves vs. what phase-20
   still needs; keep `SFU_MODE=sovereign` **gated off**.

## Non-goals (remain phase-20)

- Full trickle ICE (candidate gathering, connectivity checks), DTLS/SRTP handshake to a
  real peer, RTP `MediaData` forwarding, per-room fan-out. This spike proves the loop
  turns; it does not move media.
- Flipping `SFU_MODE=sovereign` on — it stays gated off (spike, not a shipped SFU).

## Impact

- Affected: `crates/frf-media-str0m/src/{transport_spike.rs,lib.rs}`,
  `crates/frf-media-str0m/SPIKE-FINDINGS.md`.
- The load-bearing transport-loop unknown is de-risked with a real socket + a passing test;
  the full media build (phase-20) starts from proven loop mechanics instead of a cold start.
- No library `unwrap`/`expect`; `SFU_MODE=sovereign` remains gated off — no "healthy but
  does nothing."
