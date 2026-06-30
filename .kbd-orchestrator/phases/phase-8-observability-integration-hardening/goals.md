# Goals — Phase 8: Observability, Integration Tests, and Production Hardening

> Seeded from Phase 7 reflection · 2026-06-21

## Context

Phase 7 completed the WebRTC str0m SFU adapter, WASM browser SDK depth (AgentStream + JWT),
RunAgent bidi upgrade, admin-UI transport toggle, and Dagger CI with Node 24 + WASM build stage.

The core realtime fabric is now architecturally complete from port to browser. Phase 8 focuses
on making the system production-ready: systematic observability, full-stack integration tests,
LogBroker spine wiring for remaining dev-inject routes, and Cedar policy engine scaffolding.

## Goals

1. **Full-stack E2E integration tests** — Layer 3 Playwright tests exercising the actual
   WASM subscribe → gateway → LogBroker → fan-out flow end-to-end. Requires a Dockerized
   test stack (gateway + LogBroker + SurrealDB). Wire `WASM_AVAILABLE=1` into Dagger E2E stage.

2. **Observability: tracing spans** — Add `tracing` spans across every port boundary call
   (CLAUDE.md requirement not yet systematically met). Wire span export to an OpenTelemetry
   collector. Cover: publish, subscribe, CRDT apply, authz check, federation bridge relay.

3. **LogBroker spine wiring** — Wire `/dev/inject-federation-event` to `log_broker.publish()`;
   wire the subscribe fan-out route to `log_broker.subscribe()` so Layer 3 tests can drive
   the full spine path without a live Iggy broker.

4. **`SpineSignalService` live tonic integration test** — Close the last Phase 4 debt.
   Spin up `tonic::transport::Server` in a test harness and assert bidirectional signal exchange.

5. **Performance benchmarks** — `criterion` benchmarks for `crdt_apply_delta` latency
   and subscribe fan-out throughput. Establish a baseline before further optimization.

6. **Cedar policy engine** — Scaffold `crates/frf-policy-cedar/` implementing the `AuthzProvider`
   action-policy port with Cedar 4.x. Define a sample policy set for tenant-scoped mutations.
   The Keto visibility check and the Cedar action-policy check must not be conflated.

7. **`TenantActorRegistry` sweep interval configurable** — Add `REGISTRY_SWEEP_INTERVAL_SECS`
   env var to `GatewayConfig`; eliminate the hardcoded 60s default.

## Carry-Forward Debt Entering Phase 8

| Debt Item | Source Phase | Work in Phase 8 |
|---|---|---|
| `/dev/inject-federation-event` → spine wiring | Phase 7 | Goal 3 |
| `SpineSignalService` live tonic integration test | Phase 4 | Goal 4 |
| WASM Layer 3 E2E (full-stack) | Phase 7 | Goal 1 |
| `TenantActorRegistry` sweep interval hardcoded | Phase 6 | Goal 7 |
| `ReqwestMatrixClient` federation impl | Phase 6 | Track only — unblocks on Tuwunel SDK |
| `frf-policy-cedar` crate missing | Phase 0 | Goal 6 |
