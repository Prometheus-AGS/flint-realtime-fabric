# sfu-architecture (delta)

## ADDED Requirements

### Requirement: A MediaTransport port MUST exist for the sovereign SFU media engine

`frf-ports` MUST define a `MediaTransport` trait — separate from `MediaSignaler` — that
expresses the per-session media engine (session create/answer, trickle-ICE candidate
in/out, connection-state), with a `DynMediaTransport` wrapper for runtime `SFU_MODE`
selection. The port MUST hold no implementation (the dependency rule), MUST carry ICE
candidates as `SignalEnvelope`s (consistent with signaling), and MUST NOT include RTP
forwarding (phase-21). Public enums are `#[non_exhaustive]`.

#### Scenario: the port defines the media session lifecycle without an implementation

- **WHEN** `frf-ports` is built
- **THEN** it exposes `MediaTransport` (create_session, add_remote_candidate, local_signals,
  connection_state, remove_session) and `DynMediaTransport`, with no adapter dependency

#### Scenario: RTP forwarding is not on the port yet

- **WHEN** the `MediaTransport` surface is defined
- **THEN** it stops at connection lifecycle — RTP forwarding is deferred to phase-21
