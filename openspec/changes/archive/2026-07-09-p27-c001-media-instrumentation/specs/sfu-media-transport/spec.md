# sfu-media-transport

## ADDED Requirements

### Requirement: The media negotiation path is observable during a live run
The sovereign media engine and the decode harness SHALL log the ICE/connection lifecycle so a
failed run reports where the WebRTC exchange stalled.

#### Scenario: A stalled run reports its last state
- **WHEN** the decode run fails to reach `framesDecoded > 0`
- **THEN** the harness reports the last observed ICE/connection state and the gateway logs show
  whether it accepted the offer, advertised a candidate, reached Connected, and forwarded media.
