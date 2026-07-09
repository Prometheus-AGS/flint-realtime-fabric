# p23-c004-decoded-media-proof

## Why

Phase-23 G2: prove media *decodes* end-to-end through the sovereign SFU — the proof phases
21/22 honestly gated. **Honesty finding (2026-07-08):** str0m is sans-codec — it forwards RTP,
it never decodes frames. A real decoded frame only exists inside a browser's WebRTC stack
(VP8/H264/Opus → pixels/samples), observable via `RTCPeerConnection.getStats()`
`inbound-rtp.framesDecoded > 0`. So the decode proof is **browser-side** and needs two real
browser peers through a running `SFU_MODE=sovereign` gateway.

## What Changes

- A Playwright harness `admin-ui/e2e/media-decode.spec.ts`: two Chromium contexts with fake
  media devices join the same room through the sovereign gateway (over the c003 WS inbound
  path); the receiver asserts `getStats()` reports `framesDecoded > 0` (and bytesReceived > 0).
- A browser helper `e2e/support/decode-probe.ts`: connects a peer, subscribes to the inbound
  track, and polls `getStats()` until a frame decodes or a timeout.
- **Honestly integration-gated** (`SKIP_INTEGRATION`/`GATEWAY_URL`) — with no gateway+Chromium
  it is **SKIPPED, not passed**. In this headless CI env it does not run; the harness is
  proven-*capable*, not proven-*here*.

## Impact

- New: `admin-ui/e2e/media-decode.spec.ts`, `e2e/support/decode-probe.ts`.
- No production code; **no gate flip**. Because the decode is not proven in this environment,
  c006 keeps `SFU_MODE=sovereign` OFF and re-affirms gated with this rationale.
- G2 status: harness authored + correct metric (`framesDecoded`); real run deferred to an env
  with a live sovereign gateway + Chromium media.
