# Tasks — p23-c004-decoded-media-proof

- [x] 1. `e2e/support/decode-probe.ts`: connect an `RTCPeerConnection` (recvonly) via `/ws/v1/signal`, `ontrack`, poll `getStats()` for `inbound-rtp.framesDecoded`, resolve `{decoded, framesDecoded, bytesReceived}` or timeout. No `any`.
- [x] 2. `e2e/media-decode.spec.ts`: two fake-media Chromium peers through the sovereign gateway; receiver asserts `framesDecoded > 0`. `test.skip(skipIntegration)` gate; header documents that a real decoded frame is browser-side (str0m is sans-codec) and this env skips it — no false green.
