# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: A two-peer DTLS-connected handshake MUST be proven in-process

The str0m media engine's DTLS-connected path MUST be proven by two peers completing a real
ICE connectivity check + DTLS handshake to `Connected` in-process over loopback (offerer +
answerer `Rtc`s driving real UDP), no browser. The offerer role is a test harness only — the
`MediaTransport` port stays answerer-only (the SFU always answers a browser's offer). If
loopback ICE is CI-flaky, the proof is re-gated with the local pass documented, never faked.
`SFU_MODE=sovereign` stays gated off (this proves connectivity, not media forwarding).

#### Scenario: two peers reach Connected over loopback

- **WHEN** an offerer and answerer exchange offer/answer + host candidates and drive UDP
- **THEN** both `Rtc`s reach `is_connected()` within a bounded time (real ICE + DTLS)

#### Scenario: the port stays answerer-only

- **WHEN** the offerer role is added
- **THEN** it is test-only; `MediaTransport` exposes no offerer method
