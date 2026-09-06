# ADR-008: Sovereign SFU — one shared UDP socket, demuxed by `Rtc::accepts()`

## Status

Accepted — 2026-07-09 (p29-c001)

Refines ADR-005 (the `MediaTransport` port) and ADR-006 (RTP fan-out) on **socket and driver-task ownership**. The port and room-routing
contracts are unchanged; per-session driver-task ownership is superseded.

## Scope and reconciliation — 2026-09-06

ASO integration follows [ADR-009](adr-009-aso-runtime-integration.md). Shared
socket ownership does not establish session revocation or select ASO local SQL
storage. This record refines both socket and driver-task ownership in ADR-005
and ADR-006, while preserving port and room-routing contracts.

## Context

Phase-28's live decode run reached the media exchange for the first time (the p28-c002 candidate-IP
fix), and in doing so surfaced blocker **B2**: `StrOmTransport` binds one `UdpSocket` **per session**
inside a per-session driver task (`session.rs:154`, `driver.rs::run_session`). Against a fixed media
port (`MEDIA_UDP_PORT=40000`, required so a browser can reach the socket through a mapped container
port), the **second** `create_session` fails:

```
udp bind: Address already in use (os error 98)
```

and every subsequent trickle candidate for that session hits "unknown session". A single fixed UDP
port with a per-session bind is, by construction, a **single-session** design. The decode proof
drives a two-peer room, so it can never complete under this model.

This is not a fan-out bug (ADR-006 is correct) nor a port-contract bug (ADR-005 is correct). It is a
**socket-ownership** bug: the socket is owned at the wrong granularity.

## Decision

**One shared `UdpSocket`, owned by `StrOmTransport`, demultiplexed to per-session `Rtc` instances by
`Rtc::accepts()`.** This is exactly str0m's own reference model (`examples/chat.rs:152` — "how we
demultiplex the incoming packet to know which client it belongs to" via
`clients.iter_mut().find(|c| c.accepts(&input))`).

Concretely:

- `StrOmTransport` lazily initializes **one** UDP socket from `MediaConfig`
  (bind addr + fixed port) and one owning demux task on first session creation
  through `ensure_demux()`. The synchronous constructor does not bind a socket.
  Negotiation uses the shared socket's `local_addr` for the
  advertised host candidate (the p28-c002 `resolve_advertised_ip` path is unchanged).
- A **single owning task** (the "demux loop", `demux.rs`) owns the shared socket **and** the set of
  live `Rtc`s. On each inbound datagram it finds the owning session via `rtc.accepts(&input)`, feeds
  `handle_input`, and drains that `Rtc`'s `poll_output` back to the shared socket. The loop also
  services per-session commands (`AddRemoteCandidate`, `Shutdown`), forwarded media from the
  `RoomRouter`, and each `Rtc`'s timers.
- Per-session state is unchanged in shape: each session still has its own `Rtc`, `watch`
  connection-state channel, `broadcast` local-signals channel, and `RoomRouter` forwarding channel.
  The **socket and driver task** consolidate from N to 1; the demux task owns
  every live session Rtc rather than dispatching to independent session tasks.

### Why a single owning task (not a socket-actor routing to per-session tasks)

str0m is sans-I/O and **single-threaded per `Rtc`** — a `Rtc` is driven by one thread at a time.
A single owning task that holds every `Rtc` needs **no locks** around the `Rtc` set and matches the
str0m reference exactly. A socket-actor that recv's and then forwards datagrams to N per-session
tasks would add a routing hop and a shared index both sides coordinate on, for no throughput win at
the session counts this SFU targets. (Operator decision, p29 plan: single owning task.)

## Consequences

- **Positive:** N concurrent sessions share one media port with no bind collision; the two-peer
  decode room becomes possible; the model matches str0m's guidance, so RTP demux is proven upstream.
- **Cost:** the per-session `run_session` loop is replaced by one multi-session loop. The loop owns
  all `Rtc`s, so a session is added/removed by sending it into the loop (not by spawning/aborting a
  task). File-size: the loop is split into `demux.rs` to keep every file ≤500 lines.
- **Bounded scope:** this changes only `frf-media-str0m` internals. The `MediaTransport` port, the
  gateway bridge, ADR-007 room-join authz, and the `RoomRouter` fan-out contract are untouched.
- **Fairness / head-of-line:** one task servicing all sessions must not let one session starve
  others. The loop drains bounded work per wakeup and relies on str0m's per-`Rtc` timers; RTP is
  loss-tolerant (bounded forward channels already drop on overflow, ADR-006).

## Alternatives considered

- **Per-session ephemeral ports** (each session binds port 0): avoids the collision but needs a wide
  published UDP range per container and per-session candidate ports — operationally heavier and still
  not how str0m demuxes. Rejected.
- **Socket-actor + per-session tasks:** see "Why a single owning task" — extra hop + shared index for
  no benefit at target scale. Rejected.

## References

- ADR-005 (MediaTransport port), ADR-006 (RTP fan-out), ADR-007 (media-path authz).
- str0m 0.21 `examples/chat.rs` (shared-socket `accepts()` demux); `Rtc::accepts` (`lib.rs:1885`).
- `docs/PHASE-28-DECODE-RESULT.md` (the B2 evidence).
