# ADR-006: Sovereign SFU RTP Fan-out Architecture

## Status

Proposed — 2026-07-08 (p21-c003)

Gates the RTP forwarding implementation (p21-c004). No forwarding code lands before this is
decided.

## Scope and reconciliation — 2026-09-06

The proposal status and original phase milestones below are retained as
history. [ADR-008](adr-008-shared-media-socket.md) supersedes the per-session
socket and driver-task assumptions, including the rejection of shared task
ownership on that basis. One demux task now owns all live session Rtc instances;
room routing and per-session channels remain. ASO protected lanes additionally
require [ADR-009](adr-009-aso-runtime-integration.md).

## Context

The sovereign SFU engine (`StrOmTransport`, ADR-005) runs **one isolated async driver task
per session**, each owning its own `Rtc` + tokio `UdpSocket` (`driver::run_session`). Phase-20
took a session to DTLS-connected; phase-21 must **forward RTP between peers in the same room**.

The str0m write is simple: an inbound `Event::MediaData { mid, pt, params, time, network_time,
data }` on session A becomes `rtc_b.writer(mid).write(pt, network_time, time, data)` on session
B. But A's `Rtc` and B's `Rtc` live in **different tasks** — so the load-bearing problem is not
the write, it is **getting A's `MediaData` to B's task**. This is cross-task message passing,
plus a room-membership registry to know *which* peers B forwards to.

Constraints (CLAUDE.md): the per-session-socket model (phase-20) must be preserved; no library
`unwrap`/`expect`; `SFU_MODE=sovereign` stays gated off until media actually flows.

## Decision

### Options

**Option A — central room registry in `StrOmTransport` + per-session forwarding `mpsc` (recommended).**
`StrOmTransport` holds a room registry (`DashMap<(TenantId, room) → HashSet<SessionId>>`) and,
per session, a forwarding `mpsc::Sender<ForwardedMedia>` its driver `select!`s on. When session
A's driver reads `Event::MediaData`, it looks up the room's *other* members and pushes a
`ForwardedMedia` (mid, pt, time, network_time, `Arc<[u8]>` data — cheap to clone) to each of
their forwarding channels; their drivers write it via `rtc.writer(mid).write(...)`.

- **Pros:** preserves the per-session task + socket model; the registry is one central,
  unit-testable place; room join/leave is a single registry op; `Arc<[u8]>` payload makes the
  cross-task hand-off cheap; fan-out degree is a plain lookup.
- **Cons:** one extra hop (A's driver → channel → B's driver) and a shared registry the drivers
  read; a slow consumer could back-pressure (bounded channel + drop-oldest policy needed).

**Option B — direct peer-to-peer driver channels.**
Each driver holds direct channels to its room peers' drivers, no central registry.

- **Cons:** O(n²) channel wiring; every room join/leave rewires *every* peer in the room;
  membership state is smeared across drivers. Rejected.

**Option C — shared-`Rtc` actor (one task owns all a room's `Rtc`s).**
Collapse a room into one actor task owning all member `Rtc`s so forwarding is in-task.

- **Cons:** serializes an entire room on one task (a busy room becomes a bottleneck); breaks
  the per-session-socket model phase-20 built and proved. Rejected.

### Recommendation

**Option A.** It keeps the proven per-session architecture, centralizes room membership in one
testable place, and makes forwarding an explicit, bounded cross-task hand-off.

### Shape (refined in c004)

- `StrOmTransport` gains `rooms: DashMap<(TenantId, String), HashSet<SessionId>>` and each
  `SessionHandle` a `forward_tx: mpsc::Sender<ForwardedMedia>`.
- `ForwardedMedia { mid: Mid, pt: Pt, time: MediaTime, network_time: Instant, data: Arc<[u8]> }`.
- The driver `select!`s a new `forward_rx` arm → `rtc.writer(mid).write(pt, network_time, time,
  data)`; on `Event::MediaData` it routes to the room's other members' `forward_tx`.
- Room membership: join at `create_session` (needs a room id — carried on the session), leave
  at `remove_session`.
- **pt caveat:** the write errors with `UnknownPt` if the destination `Rtc` did not negotiate
  the same payload type. Phase-21 c004 proves 1-to-1 where both peers negotiated the same codec;
  general pt-remapping is a follow-on.
- **Back-pressure:** the forwarding channel is bounded; a full channel drops the packet (RTP is
  loss-tolerant) with a metric/warn rather than blocking the source driver.

### Scope guard

c004 implements **1-to-1** forwarding on this design; **N-peer fan-out + PLI/keyframe +
renegotiation is phase-22**. `SFU_MODE=sovereign` stays gated off until media flows.

## Consequences

- **c004** implements Option A for 1-to-1 forwarding. **Phase-22** generalizes to N-peer
  fan-out + PLI on the same registry.
- The room registry is the seam where per-room fan-out (phase-22) and tenant isolation plug in.

## Related

- ADR-005 — the `MediaTransport` port + per-session engine this forwards on top of.
- [ADR-007](adr-007-media-path-authz.md) — the per-participant Keto `view` check at room-join
  that authorizes entry to this fan-out; a **G3 precondition for the `SFU_MODE=sovereign` flip**.
- p21-c002 — the two-peer DTLS-connected proof (peers must connect before media forwards).
- CLAUDE.md — per-session-socket model; no library `unwrap`/`expect`; sovereign gated off.
