# sfu-signaling

## ADDED Requirements

### Requirement: The gateway relays sovereign trickle ICE candidates to the browser
For `SFU_MODE=sovereign`, the `/ws/v1/signal` endpoint SHALL relay the media engine's local trickle
ICE candidates and connection-state to the browser, and accept the browser's inbound ICE
candidates, so ICE can complete between the browser and the SFU.

#### Scenario: Bidirectional trickle
- **WHEN** a browser negotiates an offer with the sovereign SFU
- **THEN** the browser receives the SFU's `ice-candidate` frames and the SFU receives the browser's,
  and ICE reaches `connected`.
