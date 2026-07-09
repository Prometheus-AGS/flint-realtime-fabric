# sfu-media-transport

## ADDED Requirements

### Requirement: The sovereign media transport hosts multiple concurrent sessions on one shared UDP socket
`StrOmTransport` SHALL bind a single UDP socket for the whole transport and demultiplex inbound
datagrams to the owning `Rtc` by `Rtc::accepts()`, so that N concurrent sessions share one media
port without a per-session bind collision.

#### Scenario: Two sessions on one fixed media port
- **WHEN** two sessions are created against a transport configured with a single fixed UDP port
- **THEN** both negotiate successfully with no `Address already in use` error.

#### Scenario: Inbound datagrams route to the owning session
- **WHEN** a datagram arrives for a specific session's `Rtc`
- **THEN** it is dispatched to that `Rtc` (matched by `accepts`), not another session's.
