# sfu-spike (delta)

## ADDED Requirements

### Requirement: The str0m sans-I/O UDP transport loop MUST be proven over a real socket

A spike MUST prove the str0m sans-I/O event loop turns over a **real bound UDP socket**:
binding a socket, adding its actual local address as a host candidate, and driving
`Rtc::poll_output` (handling `Transmit`/`Timeout`/`Event`) and `Rtc::handle_input`
(`Input::Receive`/`Timeout`) without panic. The full media loop (trickle ICE, DTLS/SRTP,
RTP forwarding, per-room fan-out) remains out of scope and deferred to a dedicated phase;
`SFU_MODE=sovereign` stays gated off until media actually flows.

#### Scenario: the transport loop turns over a real socket

- **WHEN** a socket is bound, an offer negotiated, and the loop polled
- **THEN** `poll_output` yields a `Transmit` or `Timeout` step (the loop turns) without error

#### Scenario: an inbound datagram is fed without panic

- **WHEN** a datagram is fed into `handle_input` as an `Input::Receive`
- **THEN** it is processed (or cleanly rejected) without panic

#### Scenario: sovereign mode stays gated off

- **WHEN** the transport-loop spike lands
- **THEN** SFU_MODE defaults to hosted and sovereign selection still moves no media
