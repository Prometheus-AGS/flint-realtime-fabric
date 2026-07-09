# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The sovereign SFU MUST forward RTP media between two connected peers

The str0m media engine MUST forward inbound `Event::MediaData` from one session to the other
member(s) of its room via a central room registry + per-session forwarding channels (ADR-006),
writing to the destination `Rtc` with `writer(mid).write`. Forwarding MUST be bounded (a full
forwarding channel drops the packet — RTP is loss-tolerant — rather than blocking the source).
Phase-21 proves **1-to-1** forwarding between two connected peers; N-peer fan-out + PLIs are
deferred (phase-22). `SFU_MODE=sovereign` stays gated off until media flows end-to-end.

#### Scenario: a media packet is forwarded from one peer to another

- **WHEN** two peers are connected and joined to the same room, and peer A writes a media frame
- **THEN** peer B receives the frame forwarded through the SFU's room router

#### Scenario: forwarding never blocks the source driver

- **WHEN** a destination's forwarding channel is full
- **THEN** the packet is dropped with a warning, not blocked (RTP is loss-tolerant)

#### Scenario: N-peer fan-out and PLI remain deferred

- **WHEN** more than two peers or keyframe handling is needed
- **THEN** it is deferred to phase-22; phase-21 proves 1-to-1 only, gate stays off
