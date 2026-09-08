# media-e2e

## ADDED Requirements

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

## MODIFIED Requirements

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
