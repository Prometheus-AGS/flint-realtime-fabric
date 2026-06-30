# p4-c005 — Admin UI signaling feature

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p4-c003-signal-grpc-service (SpineSignalService mounted and accepting connections)
p4-c004-frf-wasm (frf-wasm WASM SDK built to sdks/ts/frf-wasm/)
p2-c009-admin-ui-scaffold (admin-ui structure established)

## Crates / packages affected
`admin-ui` (TypeScript / React 19)

## Dependency-rule impact
Frontend only — no Rust crates modified. Dependency rule: N/A.

## What this change does

Adds `admin-ui/src/features/signaling/` with:

1. **`SignalingPanel` component** — displays current room state (joined/left),
   ICE connection status, and session participant count. Uses Base UI primitives.
2. **`useSignalingStream` hook** — opens a Connect-ES bidi stream to
   `SignalService.Signal`, dispatches received envelopes to the Zustand store.
3. **`signalingStore` (Zustand)** — state: `{ roomId, sfuMode, status, participants }`.
   Actions: `joinRoom`, `leaveRoom`, `onSignalFrame`.
4. **`signalingService`** — wraps `@connectrpc/connect` `createBidiStreamingClient`,
   exposes `openSignalStream(roomId): AsyncIterable<SignalEnvelope>`.
5. **Demo page** wired into `admin-ui/src/core/` routing: `/demo/signaling` — shows
   `SignalingPanel` alongside the existing `EntityStream` component to demonstrate
   entity + signaling co-existence.
6. **CRDT demo button** — calls `crdt_apply_delta` via `frf-wasm` WASM import,
   shows the merged result in a readonly textarea (proves WASM is loaded).

Component rules: components render, hooks coordinate, stores own state, services
call APIs. No component fetches data directly. No `any` types.

## Non-goals
- Does not implement a full video call UI (Phase 7)
- Does not handle LiveKit JS SDK media tracks (out of Phase 4 scope)
- Does not add auth/login UI (existing JWT header forwarding from admin-ui is sufficient)
