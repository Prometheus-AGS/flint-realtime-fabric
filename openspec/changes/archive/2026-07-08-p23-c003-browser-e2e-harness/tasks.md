# Tasks — p23-c003-browser-e2e-harness

- [x] 1. WS inbound path: `/ws/v1/signal` reads inbound browser frames, parses SDP-bearing offers/ICE (camelCase `SignalFrame` gains `sdp`/`payload`; `sfuMode` no longer hardcoded), and — for `SFU_MODE=sovereign` — drives `MediaTransportBridge`, sending the Answer back over the socket. Bridge reachable via `AppState` (sovereign-only `Option`). Rust tests for inbound parse + answer emission.
- [x] 2. `e2e/support/webrtc-client.ts`: open `/ws/v1/signal`, create `RTCPeerConnection`, send offer frame, apply answer, resolve on `connected` / reject on failed+timeout.
- [x] 3. `e2e/media-webrtc.spec.ts`: UI-shape asserts always run; the real-connection test is `test.skip(skipIntegration, …)` (SKIP_INTEGRATION/GATEWAY_URL) and asserts `connected`. Document the browser-gated boundary + Dagger flag.
