# p21-c004 — 1-to-1 RTP forwarding

## Why

ADR-006 decided the fan-out architecture (central room registry + per-session forwarding
`mpsc`). This change implements it for **1-to-1** forwarding: a media packet from peer A is
forwarded through the SFU to peer B in the same room. This is the actual media the whole
sovereign-SFU track has been building toward (still gated off until proven end-to-end).

## What Changes

Per ADR-006 (Option A):

1. **`room.rs` (new):** `ForwardedMedia { mid, pt, time, network_time, data: Arc<[u8]> }` +
   `RoomRouter` — an `Arc`-shared registry (`rooms: DashMap<(TenantId, room) → HashSet<SessionId>>`,
   `forwarders: DashMap<SessionId, mpsc::Sender<ForwardedMedia>>`). `register`/`deregister`/
   `forward(from, tenant, room, media)` push a packet to the room's *other* members (bounded
   channel; a full channel drops the packet — RTP is loss-tolerant — with a warn).
2. **`driver.rs`:** `run_session` gains a `forward_rx` arm (`writer(mid).write(pt, network_time,
   time, data)`) and, on `Event::MediaData`, calls `router.forward(..)` to route to co-room
   peers. Takes the `Arc<RoomRouter>` + the session's tenant/room.
3. **`session.rs`:** `StrOmTransport` holds the `Arc<RoomRouter>`; `create_session` registers
   the session (default room) and wires its `forward_tx`; `remove_session` deregisters. An
   inherent `join_room` (or room-aware create) groups two sessions for the 1-to-1 test —
   the `MediaTransport` port surface is unchanged.
4. **Test:** extend the c002 two-peer harness — connect A + B, join them to a room, A writes a
   media frame, assert B receives it forwarded through the router. `pt` matches (both peers
   negotiated the same codec — the ADR-006 caveat).

## Non-goals (phase-22)

- N-peer per-room fan-out, PLI/keyframe, renegotiation, general `pt` remap.
- Enabling `SFU_MODE=sovereign` (still gated off — this proves 1-to-1 media, the gate flips in
  c005 only if it flows end-to-end).

## Impact

- Affected: `crates/frf-media-str0m/src/{room.rs,driver.rs,session.rs,lib.rs}`.
- A media packet forwards A→SFU→B in-process, proven by a test. No library `unwrap`/`expect`;
  files ≤500 lines; `SFU_MODE=sovereign` stays off.
