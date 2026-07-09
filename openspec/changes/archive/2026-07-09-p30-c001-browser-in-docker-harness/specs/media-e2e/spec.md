# media-e2e

## ADDED Requirements

### Requirement: The decode harness runs the browser inside the Docker network
The decode run SHALL execute the browser on the same container network as the sovereign gateway and
coturn, so the browser's ICE candidates and the SFU's share one address space and a candidate pair
can complete (removing the host-vs-container topology split).

#### Scenario: In-network browser reaches the gateway by service name
- **WHEN** the decode harness runs inside the compose network
- **THEN** it reaches the gateway at its in-network origin and the browser has a secure context for `getUserMedia`/`RTCPeerConnection`.
