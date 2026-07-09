# p27-c003-room-join-fanout

## Why

FIND-3: both the sender and receiver send only an `offer`, so each becomes its own SFU session in
its **own default room (= its session id)**. Neither joins `e2e-decode-room`, so `RoomRouter` never
relays the sender's RTP to the receiver — even once ICE completes (c002), no media fans out.

## What Changes

- **Harness**: after the offer, both peers send a `RoomJoin` frame for the shared room so
  `MediaTransportBridge`→`StrOmTransport::join_room` regroups them into `e2e-decode-room` and the
  `RoomRouter` fans the sender's media to the receiver. The receiver's join is authorized by the
  ADR-007 Keto `view` grant (seeded by `seed-media-view.sh`).
- Applies to `decode-probe.ts` (receiver) and the `media-decode.spec.ts` sender inline block.

## Impact

- `admin-ui/e2e/support/decode-probe.ts`, `admin-ui/e2e/media-decode.spec.ts`. No production code.
- With ICE (c002) + fan-out (c003), the c004 re-run can reach `framesDecoded > 0`.
