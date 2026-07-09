# media-e2e

## ADDED Requirements

### Requirement: The decode harness offers a TURN relay so a routable candidate pair always forms
The decode run SHALL provide a TURN relay and configure both browsers' `RTCPeerConnection` with it, so
each gathers a relay candidate (a routable IP the SFU accepts and pairs) — the environment-independent
fix for a bridge topology where host/srflx candidates do not pair.

#### Scenario: Browser gathers a relay candidate the SFU pairs
- **WHEN** the decode run configures a `turn:` server with credentials
- **THEN** the browser gathers a `typ relay` candidate, the gateway accepts it, and ICE can complete.
