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

This proof has been observed passing on a local single-bridge topology with both
peers inside the container network. That result establishes that the relay
decodes real media. It SHALL NOT be read as a multi-host, NAT-traversal, or
scale result, and any claim beyond the topology actually exercised requires its
own run.

#### Scenario: Gateway + browser media present
- **WHEN** two fake-media browser peers connect through the sovereign gateway
- **THEN** the receiver's `framesDecoded` exceeds zero.

#### Scenario: No gateway / no browser media
- **WHEN** no sovereign gateway is configured
- **THEN** the decode test is skipped (not passed); nothing claims decoded media flowed.

#### Scenario: A passing decode is distinguished from a skipped one

- **WHEN** the decode proof reports success
- **THEN** the assertion observed a frame count read from live inbound-rtp statistics
- **AND** a run in which the test was skipped is not reported as a pass

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

A run that stops before the browsers are launched SHALL be distinguishable from a
run whose media path failed. A harness abort produces no probe reading at all,
and SHALL NOT be classified as an ICE or media-path outcome.

#### Scenario: A failed decode run is diagnosable
- **WHEN** the decode run fails to reach `framesDecoded > 0`
- **THEN** the assertion message reports `ice`/candidate counts and the gateway str0m logs are saved.

#### Scenario: An aborted run is not reported as a media-path result

- **GIVEN** a decode run that aborts before the browser peers are launched
- **WHEN** its outcome is classified
- **THEN** it is not reported as an ICE failure
- **AND** it is not reported as a media-path failure

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

### Requirement: The SFU SHALL advertise a host candidate the peer can pair with

The address the SFU advertises as its host candidate SHALL belong to an address
family the receiving peer can reach. Where the advertised address is configured
as a hostname, and that hostname resolves to more than one address family, the
configuration SHALL NOT rely on the resolver's ordering to select a reachable
one.

This is not a theoretical hazard. A dual-stack container bridge resolves a
service name to both an IPv4 and an IPv6 address; a resolver that returns the
IPv6 address first causes the SFU to advertise a candidate the peer never pairs
with. ICE then remains in its initial state and the decode times out having
received no bytes — a failure that reads as a media-path defect and is not one.

#### Scenario: A hostname resolving to two families does not strand ICE

- **GIVEN** an advertised-address hostname that resolves to both an IPv4 and an IPv6 address
- **AND** a peer that can reach only the IPv4 address
- **WHEN** the SFU negotiates a session
- **THEN** the advertised host candidate is one the peer can reach
- **AND** ICE does not remain in its initial state

#### Scenario: The advertised address is overridable per topology

- **WHEN** an operator supplies an explicit advertised address
- **THEN** that address is used in place of the default
- **AND** the default remains in effect when none is supplied

### Requirement: The local decode path SHALL NOT depend on environment a CI job supplied

Every input the decode runner needs SHALL be obtainable on a developer machine
by running the runner. A value that a CI job provided out of band — a secret
exported job-wide, a dependency installed by a separate step — SHALL NOT be an
unstated precondition of the local run.

Because testing is local-integration only, an input that exists solely as a CI
job step is not a gap in convenience; it makes the proof unrunnable as written.

#### Scenario: A required secret absent from the environment does not abort the run

- **GIVEN** a required secret is not set in the environment
- **WHEN** the decode runner is invoked
- **THEN** the runner supplies a value rather than aborting before any service starts

#### Scenario: A health check reaches a serving gateway

- **GIVEN** a gateway serving its health endpoint inside its container
- **AND** a published port whose loopback name resolves to more than one address family
- **WHEN** the runner polls the gateway for health
- **THEN** the poll observes the gateway as healthy
