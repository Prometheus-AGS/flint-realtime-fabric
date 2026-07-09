# p29-c001-shared-demux-socket

## Why

Phase-28 B2: `StrOmTransport` binds one `UdpSocket` **per session** (`session.rs:154`) inside a
per-session driver task. The second `create_session` on the fixed media port fails
`udp bind: Address already in use (os error 98)`, and its trickle candidates then hit
"unknown session" — a single fixed port is a single-session design and cannot host the two-peer
decode room. Every real SFU (including str0m's own `chat.rs` example) uses **one shared UDP socket**
demuxed to many `Rtc` by `Rtc::accepts()`.

## What Changes

- **ADR-008** (`docs/decisions/adr-008-shared-media-socket.md`): socket ownership moves from
  per-session to `StrOmTransport`; a single owning task holds the shared socket + all `Rtc`s and
  routes inbound datagrams by `Rtc::accepts()`. Refines ADR-005/006 (not a re-open).
- `StrOmTransport` binds **one** shared `UdpSocket` (from `MediaConfig`) at construction; `negotiate`
  stops binding and uses the shared `local_addr` for the advertised host candidate.
- Replace N per-session `run_session` tasks with **one demux loop** (`demux.rs`): `select!` over the
  shared `recv_from` → `find(|r| r.accepts(&input))` → `handle_input`; drain each `Rtc`'s
  `poll_output` back to the shared socket. Per-session state/forward/command channels + `RoomRouter`
  fan-out stay intact.
- Files split to stay ≤500 lines; no library `unwrap`/`expect`; `tracing` spans preserved.

## Impact

- `crates/frf-media-str0m/src/{session,driver,demux,config}.rs`, new
  `docs/decisions/adr-008-shared-media-socket.md`. No `MediaTransport` port-contract change (the
  gateway surface is unchanged). Unblocks the multi-session decode room (B1 + G3 follow).
