# p21-c002 — offerer role + two-peer DTLS-connected proof

## Why

Phase-20 reached the DTLS-connected *milestone* but the two-peer proof
(`two_peers_reach_dtls_connected`) was `#[ignore]`d: `create_session` is answerer-only, so
there was no offerer peer to complete ICE + DTLS with. This change adds an offerer path and
**un-`#[ignore]`s the proof** — two peers complete a real ICE connectivity check + DTLS
handshake to `Connected` in-process over loopback, no browser. This is the prerequisite the
c004 RTP-forwarding proof needs (you can't forward media between peers that never connect).

## Design

The production SFU is always the **answerer** (a browser sends the offer), so an offerer path
does not belong on the `MediaTransport` port. Instead c002 adds a **connectivity test
harness**: two `Rtc`s (offerer + answerer) + two loopback `UdpSocket`s, exchanging offer/
answer + mutual host candidates, driven by a shared shuttle loop
(`Transmit`→`send_to`→`recv_from`→`handle_input`) until both report `is_connected()`. This
exercises str0m's **real** ICE + DTLS over real sockets — the exact thing the gated test
needed — without adding a non-SFU offerer method to the port.

## What Changes

1. **`session.rs` tests:** a `connect_two_peers()` harness (offerer + answerer `Rtc`s, loopback
   sockets, SDP + candidate exchange, shuttle loop). Un-`#[ignore]`
   `two_peers_reach_dtls_connected` to drive it and assert both reach `is_connected()` within a
   bounded time.
- No production API change (the port stays answerer-only; the offerer is test-only).

## Non-goals

- Adding an offerer path to the `MediaTransport` port (the SFU is always the answerer).
- RTP forwarding (c004); enabling `SFU_MODE=sovereign` (still gated off).

## Impact

- Affected: `crates/frf-media-str0m/src/session.rs` (test module only).
- The two-peer DTLS-connected proof runs (no longer `#[ignore]`d) — real ICE + DTLS to
  `Connected` in-process. If loopback ICE proves CI-flaky, the test is re-gated `#[ignore]`
  with the local-pass documented (honest fallback).
