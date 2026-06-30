# Goals — Phase 4: WebRTC Media Plane + WASM Browser SDK

> RFC-FRF-002 · Prometheus AGS
> Seeded from: phase-3-ffi-sdks-crdt reflection
> Source: IMPLEMENTATION-PLAN.md §322 — Phase 4

## Prerequisites (must resolve at kickoff)

- **WebRTC SFU decision: str0m (sovereign) vs LiveKit (hosted)** — propagates into
  `frf-media-str0m` vs `frf-media-livekit` adapter design. Commit as an ADR before
  any signaling code is written.
- **`wasm-bindgen` + Connect-ES version compatibility** — confirm current versions
  and browser target support before `frf-wasm` scaffold begins.
- **`frf-postgres-cdc` WAL consumer completion** — HIGH debt from Phase 1+2. The WAL
  logical replication consumer loop is a stub. Must be completed so CDC-sourced entity
  events flow end-to-end. This is change 1 of this phase.

## Primary Goals

- **`frf-postgres-cdc` completion** — finish the WAL logical replication consumer
  loop; decode `pgoutput` protocol; publish decoded entity change events to the spine
  via `LogBroker`; wire into `frf-gateway`
- **`frf-wasm` browser SDK** — `wasm-bindgen` + Connect-ES; in-browser CRDT via
  optional `frf-crdt` WASM build; expose `subscribe`, `publish`, `crdt_apply_delta`
  to TypeScript
- **`frf-media-str0m` adapter** — sovereign SFU signaling via the str0m sans-I/O
  library; signaling envelopes on the spine; media never logged
- **`frf-media-livekit` adapter** — hosted conferencing signaling adapter (stub or
  full depending on SFU decision)
- **Signaling gRPC service** — bidi `Signal(stream SignalFrame) returns (stream
  SignalFrame)` RPC in `frf-gateway`; wire selected media adapter
- **Admin UI media demo** — React 19 component in `admin-ui/` showing entity stream
  + signaling state (no full video call UI — Phase 7)

## Exit Criterion (IMPLEMENTATION-PLAN §322)

> Browser client (via `frf-wasm` + Connect-ES) subscribes to an entity stream,
> edits offline, and reconnects via WebSocket mux; CDC event from PostgreSQL WAL
> flows end-to-end through the spine to the browser.

## Non-Goals for Phase 4

- Agent protocols / BossFang (Phase 5)
- Federation bridges — Matrix, ATProto (Phase 6)
- Full video call UI (Phase 7)
- SDK registry publishing (Phase 7)
- AI-voice session wiring (Phase 5 or 7)
