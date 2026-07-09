# Tasks — p22-c002-pli-keyframe-forwarding

- [x] 1. room.rs: `ForwardedFrame { Media(ForwardedMedia), KeyframeRequest(KeyframeRequest) }`; channel type → ForwardedFrame; `forward` wraps Media; add `forward_keyframe_request(from, req)`; unit-test the request routing
- [x] 2. driver.rs: on `Event::KeyframeRequest` call `router.forward_keyframe_request`; `forward_rx` arm matches ForwardedFrame → `writer.write` (Media) / `writer(mid).request_keyframe` (Keyframe, guarded)
- [x] 3. session.rs: forward channel type → `ForwardedFrame` (create_session wiring)
- [x] 4. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; files ≤500 (read the QA-gate verdict before archive)
