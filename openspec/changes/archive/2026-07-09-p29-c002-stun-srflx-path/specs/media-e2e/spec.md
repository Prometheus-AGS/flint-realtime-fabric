# media-e2e

## ADDED Requirements

### Requirement: The decode harness supplies the browser a STUN-reflexive candidate the SFU accepts
The decode run SHALL provide a STUN server and configure the browser `RTCPeerConnection` with it, so
Chrome gathers a server-reflexive candidate (a routable IP) rather than only mDNS `.local` host
candidates the SFU cannot parse.

#### Scenario: Browser gathers a routable candidate
- **WHEN** the decode probe connects with a STUN server configured
- **THEN** the gateway receives at least one parseable remote candidate and ICE can advance.

#### Scenario: Unparseable host candidate is skipped, not fatal
- **WHEN** the browser also sends an mDNS `.local` host candidate
- **THEN** the SFU skips that candidate and keeps the session alive.
