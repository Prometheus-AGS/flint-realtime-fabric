# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The str0m media transport MUST reach DTLS-connected

The str0m media engine MUST install a crypto provider so DTLS can key, and MUST expose a way
to await a session reaching `ConnectionState::Connected` (DTLS handshake complete). Reaching
`Connected` MUST be proven in-process (two sessions over loopback) where feasible, otherwise
the proof is integration-gated with the reason documented — never a faked pass. Reaching
`Connected` is the connection milestone only: **RTP forwarding is deferred to phase-21** and
`SFU_MODE=sovereign` stays gated off (connected ≠ media flowing).

#### Scenario: the engine installs a crypto provider

- **WHEN** the transport is constructed
- **THEN** a process-default crypto provider is installed so DTLS can complete

#### Scenario: a session can be awaited to Connected

- **WHEN** `wait_for_connected` is called for a session that reaches DTLS-connected
- **THEN** it resolves once the session's state is `Connected` (or times out)

#### Scenario: reaching Connected does not forward media

- **WHEN** a session reaches `Connected`
- **THEN** no RTP is forwarded (phase-21) and `SFU_MODE=sovereign` stays gated off
