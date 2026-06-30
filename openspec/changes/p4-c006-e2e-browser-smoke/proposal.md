# p4-c006 — Phase 4 exit-criterion E2E browser smoke test

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p4-c001-cdc-gateway-wiring (CDC consumer running)
p4-c004-frf-wasm (WASM SDK built)
p4-c005-admin-ui-signaling (admin UI serving /demo/signaling)

## Crates / packages affected
`tests/browser/` (new workspace-level test directory)

## Dependency-rule impact
Test-only directory; not a Cargo crate member. No impact on crate dependencies.

## What this change does

Implements the Phase 4 exit criterion:
> "Browser client (via `frf-wasm` + Connect-ES) subscribes to an entity stream,
> edits offline, and reconnects via WebSocket mux; CDC event from PostgreSQL WAL
> flows end-to-end through the spine to the browser."

The test is a Playwright browser automation test that:

1. Starts the gateway (`cargo run -p frf-gateway`) against a Docker Compose stack
   (Postgres 17 with `wal_level = logical`, Iggy, Keto stub)
2. Opens the admin UI at `http://localhost:3000/demo/signaling`
3. Asserts the entity stream widget receives at least one entity event
4. Triggers a Postgres INSERT via `psql` (simulates WAL CDC event)
5. Asserts the entity event appears in the admin UI within 5 seconds
6. Clicks the CRDT demo button, asserts merged result is displayed
7. Asserts no console errors from WASM initialization

A `docker-compose.test.yml` is provided for the test stack with minimal service
definitions (Postgres + Iggy only; Keto/Oathkeeper stubs via mock env vars).

## Non-goals
- Does not test LiveKit signaling E2E (requires LiveKit Cloud credentials)
- Does not test WebRTC media tracks (Phase 7)
- Does not run in unit test mode — this is an integration/E2E test requiring Docker
