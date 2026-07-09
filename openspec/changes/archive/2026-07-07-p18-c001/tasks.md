# Tasks — p18-c001

- [x] send_signal: deliver on `to_session` (unicast) when set; fan out to room sessions on `room_id` when `to_session` is None
- [x] Maintain a room registry (room_id → sessions) alongside the session map
- [x] Fix the masking test: subscribe as B, send from A with to_session=B, assert B receives it
- [x] Add a room-broadcast test (to_session=None → all room members)
- [x] clippy pedantic + unwrap_used clean; no file >500 lines
