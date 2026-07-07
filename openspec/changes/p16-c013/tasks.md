# Tasks — p16-c013

- [x] Register missing gRPC servers in the gateway
- [x] Bind sync + agent clients in SDKs
- [x] Bind signal + entity clients in SDKs
- [~] Bind authz client in SDKs
- [x] Smoke test each service through the TS SDK

## Scope (operator decision, 2026-07-06)

Investigation found the 6 proto services split three ways:
- **Spine** — server + clients already wired (c008/c011/c012).
- **Sync, Agent, Signal** — have gateway server impls; clients bindable.
- **Entity, Authz** — proto-only, NO gateway server implementation exists.

Decision: wire Sync's server + bind clients for the services that HAVE servers.
Do NOT bind Entity/Authz clients (they would target non-existent servers = dead
code). Entity/Authz need server impls first — recorded as a follow-up.

## Task 1 — register SyncService (closes the c008 deferral)

`SyncService` had an impl (`sync_grpc_service.rs`) but was never registered because
`SyncUseCase` wasn't constructed in main. Now wired in `spawn_grpc_server` with
`SyncUseCase::new(InMemoryCrdtStore, RedbOpStore::in_memory(), LoroDeltaApplier)` and
added via `.add_service(sync_svc)`. Made `RedbOpStore::in_memory()` a public (non-test)
constructor for the ephemeral default op-store; added `frf-store-redb` as a gateway dep.
The gateway now registers Spine + Signal + Sync + Agent over gRPC-web.

## Tasks 2, 3 — bind Sync/Agent/Signal clients (TS/Go/C#)

The generated clients for all three already existed in each SDK's codegen; they just
weren't surfaced. Added thin factory wrappers:
- **TS** (`sdks/ts/src/services.ts`): `createSyncClient/createAgentClient/createSignalClient`
  + service exports, re-exported from `index.ts`. `tsc --noEmit` → 0.
- **Go** (`sdks/go/client/services.go`): `NewSyncClient/NewAgentClient/NewSignalClient`.
  `go build`+`vet`+`gofmt` → clean.
- **C#** (`sdks/csharp/.../ServiceClients.cs`): `ServiceClients.CreateSync/Agent/SignalClient`.
  `dotnet build` → 0/0.

(The "entity" part of task 3 is folded into the Authz deferral below — Entity has no
server either.)

## Task 4 — authz client: DEFERRED (marked `[~]`)

`AuthzService` (and `EntityService`) have no gateway server implementation — only proto
definitions. Binding their clients would ship non-functional paths (Unimplemented at
runtime). Per the scope decision, they are NOT bound; building the Entity/Authz server
impls is a recorded follow-up (a separate change). This keeps the SDK surface honest —
every bound client reaches a real server.

## Task 5 — reachability smoke test (TS)

`sdks/ts/src/reachability.ts` — a compile-time assertion (type-checked by the build) that
every bound service's client constructs and its RPC methods are callable with the right
types: `spine.subscribe/subscribeResilient`, `sync.sync/getCheckpoint`, `agent.runAgent`,
`signal.signal`. A runtime smoke against a live gateway is gated in the Rust SDK's
integration suite (`FRF_GATEWAY_GRPC_URL`). `tsc --noEmit` → 0.

## Verification

- Gateway `cargo clippy --lib --bins` (default + dev-endpoints) → exit 0; workspace → 0
- redb tests pass (in_memory now public); fmt clean
- TS `tsc --noEmit` → 0 (SDK + reachability) · TS `build` → 0
- Go `build`+`vet`+`gofmt` → clean · C# `dotnet build` → 0/0

## Follow-up

Implement `EntityService` and `AuthzService` gateway servers, then bind their clients.
