# Tasks — p22-c003-gateway-sovereign-composition

- [x] 1. media_bridge.rs (new): `MediaTransportBridge` over `Arc<StrOmTransport>` + `handle(env)->Option<SignalEnvelope>` (Offer→create_session/answer, RoomJoin→join_room, IceCandidate→add_remote_candidate, RoomLeave/Hangup→remove_session); lib.rs registers it
- [x] 2. media_bridge.rs: unit tests with the real `StrOmTransport` (Offer→answer envelope; RoomJoin then peer wiring; RoomLeave→remove)
- [x] 3. main.rs: SFU_MODE=sovereign builds both StrOmSignaler + StrOmTransport; hold transport in app state; keep the "unproven end-to-end" warning; DO NOT flip the gate
- [x] 4. signal_service.rs: route inbound envelopes through the bridge when present (relay the answer); signaling relay unchanged
- [x] 5. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; files ≤500 (read the QA-gate verdict before archive)
