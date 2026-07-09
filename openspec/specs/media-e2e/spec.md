# media-e2e Specification

## Purpose
TBD - created by archiving change p23-c003-browser-e2e-harness. Update Purpose after archive.
## Requirements
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

### Requirement: A runner exercises the live decoded-media proof and records its outcome
The authenticated runner SHALL be executed against the live sovereign stack and its actual
`framesDecoded` outcome recorded as the gate-flip input, never fabricated.

#### Scenario: Authenticated live run reaches the media path
- **WHEN** the runner executes against the built + booted sovereign gateway
- **THEN** its actual result (decoded, or the concrete media-transport blocker) is recorded honestly.

### Requirement: The decode runner uses a real authenticated JWT
The decode-proof runner SHALL bring up the sovereign stack cleanly and authenticate the harness
with a real gateway-accepted JWT (not a DEV_NO_AUTH bypass), so the proof exercises the
authenticated media-authz path.

#### Scenario: Authenticated runner boots
- **WHEN** the runner runs
- **THEN** the sovereign stack (incl. flint-gate) comes up, a real JWT is obtained, the view grant
  is seeded for its subject, and the harness runs with that JWT.

### Requirement: A stalled decode run reports its diagnostics
The decode harness SHALL surface its ICE/connection diagnostics on failure (not be killed before
they print), and the runner SHALL capture the gateway's media logs before teardown.

#### Scenario: A failed decode run is diagnosable
- **WHEN** the decode run fails to reach `framesDecoded > 0`
- **THEN** the assertion message reports `ice`/candidate counts and the gateway str0m logs are saved.

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

### Requirement: The decode harness runs the browser inside the Docker network
The decode run SHALL execute the browser on the same container network as the sovereign gateway and
coturn, so the browser's ICE candidates and the SFU's share one address space and a candidate pair
can complete (removing the host-vs-container topology split).

#### Scenario: In-network browser reaches the gateway by service name
- **WHEN** the decode harness runs inside the compose network
- **THEN** it reaches the gateway at its in-network origin and the browser has a secure context for `getUserMedia`/`RTCPeerConnection`.

### Requirement: The decode runner uses a pre-built gateway image, never an in-run build
The decode runner SHALL use a pre-built gateway image and MUST NOT rebuild it during the decode run;
when the image is absent it fails fast with the out-of-band build command, so a single run cannot OOM
the host by compiling the gateway image concurrently with the stack.

#### Scenario: Pre-built image present
- **WHEN** the gateway image exists
- **THEN** the runner brings up the stack with `--no-build` and does not rebuild.

#### Scenario: Image absent
- **WHEN** the gateway image is absent (and `PREBUILD_GATEWAY` is unset)
- **THEN** the runner exits non-zero with the one-time build command, not a silent in-run rebuild.

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

### Requirement: The decode proof can run with browser and SFU on one host network
The decode harness SHALL support a host-network mode in which the gateway and the browser share a
single routable network stack, so their ICE host candidates pair directly without a bridge-network
address split.

#### Scenario: Host-net stack shares one address space
- **WHEN** the decode runs with the host-net override
- **THEN** the gateway advertises an address the (same-host-net) browser reaches, and host candidates can pair.

### Requirement: The decode harness offers a TURN relay so a routable candidate pair always forms
The decode run SHALL provide a TURN relay and configure both browsers' `RTCPeerConnection` with it, so
each gathers a relay candidate (a routable IP the SFU accepts and pairs) — the environment-independent
fix for a bridge topology where host/srflx candidates do not pair.

#### Scenario: Browser gathers a relay candidate the SFU pairs
- **WHEN** the decode run configures a `turn:` server with credentials
- **THEN** the browser gathers a `typ relay` candidate, the gateway accepts it, and ICE can complete.

### Requirement: The decode proof runs on a real Linux host with native host networking
The decode proof SHALL be runnable on a Linux host (GitHub Actions `ubuntu-latest`) where host
networking is native and containers reach the host at the docker0 bridge gateway, so the browser and
SFU share one real network stack without the same-host candidate-address confusions of the local
macOS/Colima environment.

#### Scenario: CI decode job runs the full proof
- **WHEN** the `decode-proof` workflow_dispatch job runs on `ubuntu-latest`
- **THEN** it builds the gateway image, boots the sovereign stack, runs the Playwright decode, and asserts `framesDecoded > 0`.

