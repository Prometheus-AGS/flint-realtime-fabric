# p27-c001-media-instrumentation

## Why

Phase-26 reached the media exchange but the WebRTC decode timed out with no visibility into *where*
it stalled — the str0m driver updates `ConnectionState` but does not log it, and the harness reports
only a terminal `reason` string. Before fixing the three diagnosed defects (ICE exchange, WS
candidate relay, RoomJoin fan-out), add the instrumentation that confirms the stall and verifies
each fix.

## What Changes

- **str0m driver** (`crates/frf-media-str0m/src/driver.rs`): `info!`-level tracing at the media
  lifecycle — offer accepted / host candidate advertised / ICE state change / first `MediaData`
  received / forward-to-room — so a run shows the gateway side.
- **Harness** (`admin-ui/e2e/support/{webrtc-client,decode-probe}.ts`): log `iceConnectionState`/
  `connectionState` transitions + gathered/received candidate counts; on timeout, report the **last
  observed state** (no candidates / stuck `checking` / no track) instead of a bare "timeout".

## Impact

- `crates/frf-media-str0m/src/driver.rs` (tracing only — no behaviour change),
  `admin-ui/e2e/support/{webrtc-client,decode-probe}.ts`. No production behaviour change; no flip.
