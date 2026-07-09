# media-e2e

## ADDED Requirements

### Requirement: The decode harness reaches the gateway over a genuine secure (HTTPS) origin
The decode run SHALL front the sovereign gateway with a TLS terminator so the browser reaches an
`https://` origin (and `wss://` signaling), giving it a genuine secure context in which
`navigator.mediaDevices`/`getUserMedia` are available — without relying on Chromium unsafe-origin
flags.

#### Scenario: Secure context available to the browser
- **WHEN** the decode harness runs against the TLS-fronted gateway
- **THEN** `navigator.mediaDevices` is defined and `getUserMedia` can acquire a track.

#### Scenario: WebSocket signaling over the TLS front
- **WHEN** the browser opens `/ws/v1/signal`
- **THEN** the TLS front proxies the WebSocket upgrade to the gateway and signaling proceeds.
