# media-e2e

## ADDED Requirements

### Requirement: A real browser peer connects to the sovereign SFU
The system SHALL provide an E2E harness that connects a real browser `RTCPeerConnection` to a
running `SFU_MODE=sovereign` gateway and observes ICE reaching `connected`, gated so it never
reports success without a live gateway.

#### Scenario: Gateway present
- **WHEN** `SKIP_INTEGRATION=false` and `GATEWAY_URL` point at a sovereign gateway
- **THEN** the harness drives offer/answer/ICE and asserts `iceConnectionState === "connected"`.

#### Scenario: No gateway
- **WHEN** no gateway is configured
- **THEN** the connection test is skipped (not passed) and only UI-shape assertions run.
