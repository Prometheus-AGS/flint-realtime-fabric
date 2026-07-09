# sfu-media-transport Specification

## Purpose
TBD - created by archiving change p20-c003-str0m-async-session-loop. Update Purpose after archive.
## Requirements
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

### Requirement: The str0m media transport MUST wire trickle ICE both directions

The str0m `MediaTransport` MUST accept inbound trickle-ICE candidates (extract the candidate
from a `SignalKind::IceCandidate` `SignalEnvelope` and feed `add_remote_candidate`) and MUST
expose an outbound `local_signals` stream carrying the session's local host candidate and its
connection-state changes as `SignalEnvelope`s. The envelope↔candidate mapping MUST be
unit-tested without a socket. Live srflx gathering + connectivity checks are out of scope
(STUN/browser-gated) and MUST be documented as such; RTP forwarding remains phase-21 and
`SFU_MODE=sovereign` stays gated off.

#### Scenario: an inbound candidate envelope feeds the session

- **WHEN** an `IceCandidate` envelope is passed to `add_remote_candidate`
- **THEN** the candidate string is extracted and fed to the session's `Rtc` (no panic on a
  malformed candidate — it is skipped with a warning)

#### Scenario: local_signals yields the host candidate and state changes

- **WHEN** a session is created and `local_signals` is subscribed
- **THEN** the stream yields the session's local host candidate as an `IceCandidate` envelope
  and connection-state changes as envelopes

#### Scenario: live connectivity is browser/STUN-gated

- **WHEN** srflx gathering / a real connectivity check is required
- **THEN** it is documented as browser/STUN-gated, not faked in a unit test

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

### Requirement: The str0m media engine MUST keep its modules under the file-size limit

The str0m media engine's source files MUST each stay under the 500-line limit; the per-session
async driver loop MUST live in its own module (`driver.rs`) separate from the `StrOmTransport`
/ `MediaTransport` surface (`session.rs`). The split MUST be behavior-preserving — the existing
tests pass unchanged.

#### Scenario: the driver loop is a separate module under the size limit

- **WHEN** the str0m crate is built
- **THEN** `session.rs` and `driver.rs` are each under 500 lines and the driver loop lives in
  `driver.rs`

#### Scenario: the split changes no behavior

- **WHEN** the existing `frf-media-str0m` tests run after the split
- **THEN** they pass unchanged (no behavior or API change)

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

### Requirement: The sovereign SFU MUST forward RTP media between two connected peers

The str0m media engine MUST forward inbound `Event::MediaData` from one session to the other
member(s) of its room via a central room registry + per-session forwarding channels (ADR-006),
writing to the destination `Rtc` with `writer(mid).write`. Forwarding MUST be bounded (a full
forwarding channel drops the packet — RTP is loss-tolerant — rather than blocking the source).
Phase-21 proves **1-to-1** forwarding between two connected peers; N-peer fan-out + PLIs are
deferred (phase-22). `SFU_MODE=sovereign` stays gated off until media flows end-to-end.

#### Scenario: a media packet is forwarded from one peer to another

- **WHEN** two peers are connected and joined to the same room, and peer A writes a media frame
- **THEN** peer B receives the frame forwarded through the SFU's room router

#### Scenario: forwarding never blocks the source driver

- **WHEN** a destination's forwarding channel is full
- **THEN** the packet is dropped with a warning, not blocked (RTP is loss-tolerant)

#### Scenario: N-peer fan-out and PLI remain deferred

- **WHEN** more than two peers or keyframe handling is needed
- **THEN** it is deferred to phase-22; phase-21 proves 1-to-1 only, gate stays off

### Requirement: The room router MUST fan out media to all other room members

`RoomRouter::forward` MUST deliver a frame from one session to **every other** member of its
room (not just one), and never to the sender. This N-peer fan-out MUST be proven by a test with
3+ members. RID/simulcast handling is out of scope until a real multi-quality stream needs it.

#### Scenario: a frame fans out to all other room members

- **WHEN** three sessions are in a room and one forwards a frame
- **THEN** the other two each receive it and the sender does not

### Requirement: The SFU MUST forward keyframe requests from receivers to senders

The sovereign SFU MUST relay an inbound `Event::KeyframeRequest` from one session to the room's
other member(s) and apply it via `Writer::request_keyframe` (guarded by
`is_request_keyframe_possible`), so a desynced/late peer can be served a keyframe. The request
travels receiver→sender (opposite to media) over the same per-session forwarding channel
(carrying a `ForwardedFrame` enum). Renegotiation and the real "peer decodes after PLI" proof
are browser-gated and deferred; `SFU_MODE=sovereign` stays gated off.

#### Scenario: a keyframe request routes to the other room member(s)

- **WHEN** a session forwards a keyframe request in a room
- **THEN** the other member(s) receive it (not the requester)

#### Scenario: an unsupported keyframe kind is skipped safely

- **WHEN** `request_keyframe` is not possible for a session/kind
- **THEN** it is skipped without panic (guarded by `is_request_keyframe_possible`)

### Requirement: The gateway MUST drive the sovereign media transport from the signal path

For `SFU_MODE=sovereign` the gateway MUST compose both the signaling relay (`StrOmSignaler`)
and the media engine (`StrOmTransport`), and MUST drive the `MediaTransport` from the signal
path via a `MediaTransportBridge`: `Offer`→`create_session` (relay the answer),
`RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`, `RoomLeave`/`Hangup`→
`remove_session`. The bridge mapping MUST be unit-tested. Composing the media plane MUST NOT
flip the gate: `SFU_MODE=sovereign` stays gated off (media present, end-to-end proof
browser-gated) and hosted remains the production media path. The signal path's JWT boundary is
unchanged.

#### Scenario: an offer creates a media session and relays the answer

- **WHEN** a `SignalKind::Offer` envelope reaches the bridge for sovereign mode
- **THEN** `create_session` runs and an `Answer` envelope is produced to relay outbound

#### Scenario: composing the media plane does not flip the gate

- **WHEN** the gateway composes `StrOmSignaler` + `StrOmTransport` for sovereign mode
- **THEN** `SFU_MODE=sovereign` stays gated off (unproven end-to-end); the flip is deferred

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

### Requirement: The media negotiation path is observable during a live run
The sovereign media engine and the decode harness SHALL log the ICE/connection lifecycle so a
failed run reports where the WebRTC exchange stalled.

#### Scenario: A stalled run reports its last state
- **WHEN** the decode run fails to reach `framesDecoded > 0`
- **THEN** the harness reports the last observed ICE/connection state and the gateway logs show
  whether it accepted the offer, advertised a candidate, reached Connected, and forwarded media.

### Requirement: The diagnosed media-transport defect is fixed
The sovereign media path SHALL be fixed per the diagnosed defect so the WebRTC exchange can progress
toward a decoded frame; the diagnosis and fix are recorded.

#### Scenario: Evidence-driven fix
- **WHEN** the surfaced diagnostics identify why the exchange stalls
- **THEN** the identified defect is fixed and recorded in docs/PHASE-28-DIAGNOSIS.md.

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

