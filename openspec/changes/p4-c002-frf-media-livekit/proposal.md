# p4-c002 — frf-media-livekit: LiveKit hosted SFU adapter

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p2-c007-gateway-tonic-service (tonic gRPC infrastructure in gateway established)
p3-c007-sync-grpc-service (bidi streaming pattern established)

## Crates affected
`frf-ports` (new `MediaSignaling` port trait — semver minor bump), `frf-media-livekit` (new crate)

## Dependency-rule impact
- `frf-ports`: adds `MediaSignaling` trait — Layer 1 port, no adapter deps. Semver impact: 0.1.0 → 0.2.0 (new public trait).
- `frf-media-livekit`: new Layer 2 adapter implementing `MediaSignaling` via LiveKit Cloud API. Imports `frf-ports` and `frf-domain` only. Dependency rule: safe.

## SFU Decision (ADR recorded)
**LiveKit (hosted)** selected. `SFU_MODE_HOSTED = 2` in `signal.proto`. `frf-media-str0m` is out of Phase 4 scope.

## What this change does

1. Adds `MediaSignaling` port trait to `frf-ports`:
   - `async fn join_room(room: RoomRef) -> Result<(), PortError>`
   - `async fn leave_room(room_id: &str) -> Result<(), PortError>`
   - `async fn relay_signal(envelope: SignalEnvelope) -> Result<(), PortError>`

2. Creates new crate `crates/frf-media-livekit/` implementing `MediaSignaling`:
   - Uses `livekit` crate (LiveKit Rust SDK) for room management
   - `relay_signal` maps `SignalEnvelope` fields to LiveKit data messages
   - `SFU_MODE_HOSTED` is the only mode this adapter responds to
   - All room IDs are namespaced by `tenant_id` to enforce tenant isolation

3. Adds `frf-media-livekit` to workspace `members` in root `Cargo.toml`

## Non-goals
- Does not implement str0m sovereign SFU adapter (future phase if ever needed)
- Does not handle media tracks, codec negotiation, or recording (LiveKit SDK handles)
- Does not store `SignalEnvelope` contents — signaling envelopes are never persisted
