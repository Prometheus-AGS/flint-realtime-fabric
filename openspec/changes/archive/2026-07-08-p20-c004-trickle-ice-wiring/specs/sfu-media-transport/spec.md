# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The str0m media transport MUST wire trickle ICE both directions

The str0m `MediaTransport` MUST accept inbound trickle-ICE candidates (extract the candidate
from a `SignalKind::IceCandidate` `SignalEnvelope` and feed `add_remote_candidate`) and MUST
expose an outbound `local_signals` stream carrying the session's local host candidate and its
connection-state changes as `SignalEnvelope`s. The envelope↔candidate mapping MUST be
unit-tested without a socket. Live srflx gathering + connectivity checks are out of scope
(STUN/browser-gated) and MUST be documented as such; RTP forwarding remains phase-21 and
`SFU_MODE=sovereign` stays gated off.

#### Scenario: an inbound candidate envelope feeds the session

- **WHEN** an `IceCandidate` envelope is passed to `add_remote_candidate`
- **THEN** the candidate string is extracted and fed to the session's `Rtc` (no panic on a
  malformed candidate — it is skipped with a warning)

#### Scenario: local_signals yields the host candidate and state changes

- **WHEN** a session is created and `local_signals` is subscribed
- **THEN** the stream yields the session's local host candidate as an `IceCandidate` envelope
  and connection-state changes as envelopes

#### Scenario: live connectivity is browser/STUN-gated

- **WHEN** srflx gathering / a real connectivity check is required
- **THEN** it is documented as browser/STUN-gated, not faked in a unit test
