# media-e2e

## ADDED Requirements

### Requirement: The decode proof can run with browser and SFU on one host network
The decode harness SHALL support a host-network mode in which the gateway and the browser share a
single routable network stack, so their ICE host candidates pair directly without a bridge-network
address split.

#### Scenario: Host-net stack shares one address space
- **WHEN** the decode runs with the host-net override
- **THEN** the gateway advertises an address the (same-host-net) browser reaches, and host candidates can pair.
