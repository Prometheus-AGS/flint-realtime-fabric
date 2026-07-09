# sfu-architecture (delta)

## ADDED Requirements

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
