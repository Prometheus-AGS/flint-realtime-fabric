# Tasks — p21-c004-rtp-forwarding-1to1

- [x] 1. room.rs (new): `ForwardedMedia` + `RoomRouter` (rooms + forwarders DashMaps; register/deregister/forward with bounded-channel drop-on-full)
- [x] 2. driver.rs: `run_session` gains a `forward_rx` arm (writer(mid).write) + `Arc<RoomRouter>` + tenant/room; on `Event::MediaData` call `router.forward(..)`
- [x] 3. session.rs: `StrOmTransport` holds `Arc<RoomRouter>`; create_session registers + wires forward_tx; remove_session deregisters; inherent `join_room` for grouping; lib.rs registers `room`
- [x] 4. test: extend the two-peer harness — connect + join a room, A writes media, assert B receives it forwarded
- [x] 5. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; files ≤500 lines
