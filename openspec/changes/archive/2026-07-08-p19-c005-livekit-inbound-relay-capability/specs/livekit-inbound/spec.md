# livekit-inbound (delta)

## ADDED Requirements

### Requirement: The LiveKit adapter MUST provide a cross-node inbound relay capability

The LiveKit adapter MUST provide an inbound relay that forwards server-originated data-channel
payloads into the per-session signal streams, so a signal published on another node can be
re-surfaced locally. The relay MUST be built against a `LiveKitDataSource` seam so its
forwarding logic is unit-testable without a live server, and the libwebrtc-backed data source
MUST be gated behind an off-by-default cargo feature so the default gateway build stays light.
Malformed inbound payloads MUST be skipped (logged), never panic.

#### Scenario: inbound payloads are forwarded to the session stream

- **WHEN** the relay reads a valid signal payload from the data source
- **THEN** the deserialized `SignalEnvelope` is forwarded into the subscribed session streams

#### Scenario: malformed inbound payload is skipped

- **WHEN** the relay reads a payload that is not a valid `SignalEnvelope`
- **THEN** it is skipped with a warning and the loop continues (no panic)

#### Scenario: the live data source is feature-gated

- **WHEN** the default (no-`realtime`-feature) crate is built
- **THEN** no libwebrtc dependency is pulled in, and the relay seam is still present and tested
