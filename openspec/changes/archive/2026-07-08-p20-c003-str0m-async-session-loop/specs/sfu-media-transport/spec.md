# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The str0m adapter MUST drive a per-session async media loop

`frf-media-str0m` MUST implement `MediaTransport` with a per-session **async** task: bind a
tokio `UdpSocket`, negotiate the offer→answer, and drive the str0m sans-I/O loop
(`poll_output` → `send_to`/state-update; `select!` over a timer and `recv_from` →
`handle_input`) under tokio. Connection-state changes MUST be surfaced (e.g. via a watch),
and `remove_session` MUST tear the task down. RTP forwarding is out of scope (phase-21) and
`SFU_MODE=sovereign` stays gated off (the engine reaches connected, not media-forwarded).
No library `unwrap`/`expect`.

#### Scenario: a session's async loop turns over a real socket

- **WHEN** `create_session` is called with a valid offer
- **THEN** a tokio UDP socket is bound, the driver task spawned, and the loop turns (an
  outbound transmit is produced or the connection state advances off `New`)

#### Scenario: removing a session tears down its task

- **WHEN** `remove_session` is called
- **THEN** the session's driver task stops and its handle is deregistered

#### Scenario: no media is forwarded yet

- **WHEN** the async loop runs
- **THEN** it reaches connection lifecycle only; RTP forwarding is deferred to phase-21
