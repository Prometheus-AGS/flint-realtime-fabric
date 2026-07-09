# sfu-spike Specification

## Purpose
TBD - created by archiving change p18-c006. Update Purpose after archive.
## Requirements
### Requirement: The str0m negotiation round-trip MUST be proven before the full SFU build

The str0m dependency MUST be current (0.21), and a spike MUST prove the `Rtc` negotiation
round-trip (accept an SDP offer, produce a valid SDP answer) with an automated test. The
live UDP/ICE/DTLS media loop is out of scope for the spike and remains documented as
deferred; `SFU_MODE=sovereign` stays gated off until media actually flows.

#### Scenario: a real offer negotiates to a valid answer

- **WHEN** a real SDP offer is passed to the negotiation spike
- **THEN** a well-formed SDP answer is produced carrying the offered media line

#### Scenario: an invalid offer is rejected

- **WHEN** a malformed SDP is passed
- **THEN** the spike returns an InvalidOffer error

#### Scenario: sovereign mode stays gated off

- **WHEN** the spike lands
- **THEN** SFU_MODE defaults to hosted and sovereign selection still warns that no media flows

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

