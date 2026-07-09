# p22-c001 — prove N-peer per-room fan-out

## Why

`RoomRouter::forward` already fans a frame to **all** other room members (it iterates
`members.iter().filter(|&&s| s != from)`), but phase-21 only proved the **2-peer** case. This
change proves N-peer delivery — 3+ sessions in a room, one sends, all others receive — closing
the G1 gap with a test, no fan-out code change.

## What Changes

Test only.

1. **`room.rs` tests:** an N-peer test — register 3 sessions in one room; `forward` from one;
   assert the **two other** members each receive the frame and the sender does not.

## Non-goals

- Any fan-out code change (the router is already general).
- RID/simulcast handling (YAGNI until a real multi-quality stream needs it).

## Impact

- Affected: `crates/frf-media-str0m/src/room.rs` (test module only).
- N-peer fan-out delivery is proven; the router's generality is confirmed for phase-22.
