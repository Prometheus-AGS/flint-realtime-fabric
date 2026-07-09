# media-e2e

## ADDED Requirements

### Requirement: A receiver decodes media relayed by the sovereign SFU
The system SHALL provide an E2E harness in which a real browser peer receiving a track relayed
by a `SFU_MODE=sovereign` gateway observes `RTCPeerConnection.getStats()`
`inbound-rtp.framesDecoded > 0`, gated so it never reports success without a live gateway.

#### Scenario: Gateway + browser media present
- **WHEN** two fake-media browser peers connect through the sovereign gateway
- **THEN** the receiver's `framesDecoded` exceeds zero.

#### Scenario: No gateway / no browser media
- **WHEN** no sovereign gateway is configured
- **THEN** the decode test is skipped (not passed); nothing claims decoded media flowed.
