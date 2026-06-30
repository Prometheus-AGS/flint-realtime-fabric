# Goals — Phase 7: WebRTC str0m SFU + WASM SDK Depth

> Seeded from Phase 6 reflection · 2026-06-21

## Context

Phase 4 delivered the initial WebRTC + WASM layer: `frf-media-livekit` adapter,
`SpineSignalService` bidi gRPC, `frf-wasm` with `crdt_apply_delta` export, and
the admin-ui signaling panel. Phase 6 delivered federation bridges and the
server-streaming `AgentService.RunAgent` gRPC endpoint.

Phase 7 closes the remaining gaps in the WebRTC and browser transport tiers:

- The sovereign SFU (`frf-media-str0m`) was stubbed in Phase 4 — no actual
  ICE/DTLS negotiation is implemented.
- `frf-wasm` exports only `crdt_apply_delta`; `publish`, `subscribe`, and
  `AgentStream` are not yet exposed to TypeScript via wasm-bindgen.
- `AgentService.RunAgent` is server-streaming only — the client-streaming
  (bidi) upgrade was deferred from Phase 6.
- The Dagger CI pipeline does not run admin-ui builds or Layer 1 Playwright
  tests; Node 24 is not enforced in CI.
- The `/dev/inject-federation-event` endpoint does not propagate events to
  the spine, blocking Layer 3 federation smoke tests.

## Goals

1. **`frf-media-str0m` sovereign SFU** — implement `str0m`-based SFU signaling
   adapter behind the `MediaSignaler` port; ICE candidate exchange, DTLS
   negotiation handshake, and media session lifecycle management.

2. **`frf-wasm` browser SDK depth** — expand wasm-bindgen surface to expose
   `publish(envelope)`, `subscribe(tenant_id) → AsyncIterator<EventEnvelope>`,
   and `AgentStream.open(token) → AsyncIterator<AgentEvent>` to TypeScript;
   update `build_wasm.sh` to produce the npm-compatible package.

3. **`RunAgent` bidi upgrade** — extend `AgentGrpcService` from
   server-streaming to full bidi streaming; wire client-side message sends
   back to the `AgentEventBus` via LibreFangBus publish.

4. **Admin-UI WebRTC + WASM integration** — wire `frf-wasm` npm bundle into
   admin-ui; update signaling panel to use sovereign str0m SFU when configured;
   add `AgentStream` consumer to agent activity panel using the WASM SDK.

5. **Dagger CI: admin-ui + Node 24** — add Dagger pipeline for admin-ui
   (`pnpm build`, `pnpm typecheck`, `SKIP_INTEGRATION=true playwright test`);
   enforce Node 24 in the pipeline; wire `build_wasm.sh` into the Dagger build.

6. **Federation dev injection spine wiring** — wire the
   `/dev/inject-federation-event` handler to actually publish the injected
   event to the `LogBroker` spine, enabling Layer 3 federation smoke tests to
   run against a real in-process bus.

7. **Phase exit criterion** — a browser TypeScript caller uses the `frf-wasm`
   SDK to subscribe and receive a `EventEnvelope` from the gateway; confirmed
   by a Playwright test that imports the WASM bundle and asserts the message
   arrives.

## Carry-Forward Debt Entering Phase 7

| Debt Item | Source Phase | Work in Phase 7 |
|---|---|---|
| `ReqwestMatrixClient` stub | Phase 6 | Track only; unblocks when Tuwunel SDK stabilizes |
| `TenantActorRegistry` idle TTL hardcoded | Phase 6 | Make configurable via `GatewayConfig` |
| `/dev/inject-federation-event` no spine propagation | Phase 6 | Wire to `LogBroker` (Goal 6) |
| Dagger CI not enforcing Node 24 | Phase 6 | Goal 5 |
| `frf-wasm/build_wasm.sh` not in Dagger CI | Phase 4 | Goal 5 |
| `SpineSignalService` no live tonic integration test | Phase 4 | Address in Goal 3 or separate |
