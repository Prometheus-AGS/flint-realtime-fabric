# sfu-architecture Specification

## Purpose
TBD - created by archiving change p20-c001-media-transport-port-adr. Update Purpose after archive.
## Requirements
### Requirement: The sovereign SFU media plane MUST be decided by an ADR before implementation

The sovereign SFU media plane MUST NOT be implemented until an ADR names the port
architecture and it is recorded, because the media engine (per-session `Rtc` + UDP + RTP)
does not fit the signaling-only `MediaSignaler` port. The ADR MUST present the options (a new
`MediaTransport` port vs. extending `MediaSignaler` vs. an internal engine), recommend one,
and keep `SFU_MODE=sovereign` gated off until media forwards.

#### Scenario: the port decision is recorded before any engine code

- **WHEN** the sovereign media plane is planned
- **THEN** an ADR states the constraint, presents the options, and recommends one
- **AND** no per-session `Rtc`/RTP engine is implemented before the ADR

#### Scenario: the recommendation honors one-port-per-adapter

- **WHEN** the ADR recommends a media-plane home
- **THEN** it keeps signaling and media transport as separate ports (str0m implementing two
  distinct ports), not one overloaded port

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

### Requirement: The sovereign SFU RTP fan-out MUST be decided by an ADR before implementation

RTP forwarding MUST NOT be implemented until an ADR names the fan-out architecture, because
the per-session engine runs isolated driver tasks so forwarding is cross-task message passing.
The ADR MUST present the options (central room registry + per-session forwarding channels vs.
direct peer channels vs. a shared-`Rtc` actor), recommend one, document the `pt`-negotiation
caveat and back-pressure policy, and scope the first implementation to 1-to-1 forwarding
(N-peer + PLI deferred). `SFU_MODE=sovereign` stays gated off.

#### Scenario: the fan-out decision is recorded before forwarding code

- **WHEN** RTP forwarding is planned
- **THEN** an ADR states the cross-task constraint, presents the options, and recommends one

#### Scenario: the recommendation preserves the per-session model

- **WHEN** the ADR recommends a fan-out home
- **THEN** it keeps the per-session task + socket model (not a shared-`Rtc` actor) and centralizes
  room membership in one place

