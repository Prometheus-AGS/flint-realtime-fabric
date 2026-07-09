# Tasks — p27-c001-media-instrumentation

- [x] 1. Add str0m driver lifecycle `info!` tracing (offer/host-candidate/ICE-state/first-MediaData/forward). No behaviour change; clippy/fmt clean.
- [x] 2. Add harness ICE-state + candidate-count logging + last-observed-state on timeout in webrtc-client.ts/decode-probe.ts. Typecheck; no `any`. QA gate.
