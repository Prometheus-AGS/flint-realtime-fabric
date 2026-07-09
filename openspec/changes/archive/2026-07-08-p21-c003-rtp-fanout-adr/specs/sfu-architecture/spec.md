# sfu-architecture (delta)

## ADDED Requirements

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
