# Tasks — p27-c002-trickle-ice-over-ws

- [x] 1. Gateway: relay `MediaTransport::local_signals(session)` (trickle candidates + connection-state) out over `/ws/v1/signal` as `ice-candidate` frames for sovereign sessions; bridge exposes the transport's local_signals. Rust tests.
- [x] 2. Harness: `onicecandidate` → send `ice-candidate` up; inbound `ice-candidate` → `addIceCandidate` (+remoteCandidates). Typecheck. QA gate.
