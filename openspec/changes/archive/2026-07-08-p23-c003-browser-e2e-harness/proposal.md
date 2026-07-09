# p23-c003-browser-e2e-harness

## Why

Phase-23 G1: prove a **real browser peer** connects to the sovereign SFU. The media plane is
composed + layer-proven (phases 21–22) but no real `RTCPeerConnection` has ever reached
`Connected` through the sovereign gateway. This change adds that harness — honestly
integration-gated so it never fabricates a pass when no gateway is running.

## What Changes

> Scope widened (operator decision, 2026-07-08): the `/ws/v1/signal` route was outbound-only; a real browser proof needs the **inbound WS→bridge path**. Task 1 adds it so the browser drives the SFU over its natural WS transport.

- A Playwright spec `admin-ui/e2e/media-webrtc.spec.ts` that, **when a live `SFU_MODE=sovereign`
  gateway is present** (`SKIP_INTEGRATION=false` + `GATEWAY_URL`), drives a real browser
  `RTCPeerConnection`: create offer → send over the `/ws/v1/signal` WebSocket → apply the SFU's
  answer → assert ICE reaches `connected`. Mirrors the existing layer2 `SKIP_INTEGRATION` gate:
  with no gateway it runs only the UI-shape assertions and **skips** the connection test (no
  false green).
- A small browser-side helper (`e2e/support/webrtc-client.ts`) that opens the signal WebSocket,
  performs offer/answer + ICE, and resolves when `iceConnectionState === "connected"` (or
  rejects on failure/timeout) — reusable by c004.
- Dagger wiring note: the harness runs behind a browser-available flag; where CI lacks Chromium
  it is documented as locally-run (honest), not silently skipped.

## Impact

- New: `admin-ui/e2e/media-webrtc.spec.ts`, `admin-ui/e2e/support/webrtc-client.ts`.
- No production code change; no gate flip. `SFU_MODE=sovereign` stays off.
- G2 (decoded-media proof, p23-c004) builds on this harness.
