# sfu-media-transport

## ADDED Requirements

### Requirement: The media transport bind address and advertised candidate are configurable
The sovereign media transport SHALL accept a configurable bind address, advertised host-candidate
IP, and UDP port, so a browser peer can reach it across process/container boundaries; absent
configuration it defaults to loopback-ephemeral.

#### Scenario: Configured advertise IP
- **WHEN** `StrOmTransport::with_config` is given an `advertise_ip`
- **THEN** the SDP host candidate advertises that IP (not loopback).

#### Scenario: Default is loopback
- **WHEN** `StrOmTransport::new()` is used
- **THEN** it binds loopback-ephemeral as before (tests unaffected).
