# p4-c003 — SpineSignalService: bidi gRPC signaling in frf-gateway

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p4-c002-frf-media-livekit (MediaSignaling port + LiveKit adapter)
p2-c007-gateway-tonic-service (tonic mounted in gateway)

## Crates affected
`frf-gateway`

## Dependency-rule impact
`frf-gateway` already imports adapters at the interface layer. Adding
`frf-media-livekit` as a gateway dependency is consistent with the composition
rule. No changes to domain or port crates. Dependency rule: safe.

## What this change does

Implements the bidi `Signal(stream SignalEnvelope) returns (stream SignalEnvelope)` RPC
defined in `proto/flint/v1/signal.proto` (frozen at `proto-v1`):

1. Creates `SpineSignalService` struct in `frf-gateway` implementing tonic's
   generated `SignalServiceServer` trait
2. The handler:
   - Validates JWT via `IdentityVerifier` (gateway boundary — never trust downstream)
   - Checks tenant membership via `AuthzProvider` (Keto)
   - For each inbound `SignalEnvelope`: routes to `MediaSignaling::relay_signal`
   - Fan-out: publishes the `SignalEnvelope` onto the Iggy spine as a
     `EventKind::Signal` envelope; spine consumers deliver to subscriber sessions
3. Extends `AppState` with `Arc<dyn MediaSignaling + Send + Sync>` field
4. Mounts `SpineSignalService` on the gateway via `axum::Router::route_service`
   using the tonic–axum composition pattern (`tonic_web::enable`)

JWT is verified at the gateway boundary per the security constraint — `SignalEnvelope`
downstream of the handler carries pre-verified tenant/session identity.

## Non-goals
- Does not implement ICE server rotation or TURN credential vending (Phase 7)
- Does not record or store SignalEnvelope contents (signaling is never persisted)
- Does not implement room-level authorization beyond tenant membership (Phase 5 Cedar policy)
