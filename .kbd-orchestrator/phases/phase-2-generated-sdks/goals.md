# Goals — Phase 2: Generated SDKs + entity-management adapter

> RFC-FRF-002 · Prometheus AGS
> Source: IMPLEMENTATION-PLAN.md §08

## Primary Goals

- Generate the **Go SDK** (`sdks/go/`) from frozen `proto-v1` using `protoc` + `protoc-gen-go` + `protoc-gen-connect-go`
- Generate the **C# SDK** (`sdks/csharp/`) from frozen `proto-v1` using `protoc` + `Grpc.Tools`
- Generate the **browser-TS SDK** (`sdks/ts/`) from frozen `proto-v1` using `buf` + `@connectrpc/connect` (Connect-ES)
- Implement the **`entity-management` RealtimeAdapter** (`sdks/entity-management/`) as a thin TypeScript wrapper on the TS SDK; connects `prometheus-entity-management` entities to the realtime fabric
- Wire all four SDKs against the live `frf-gateway` for an end-to-end smoke test

## Prerequisite (carried from Phase 1 gate)

- **Fix `PublishUseCase` missing Keto authz check** (HIGH debt from Phase 1 reflection §4): a tenant can currently publish without a write-permission check; must be resolved before Phase 2 publish-path clients arrive

## Exit Criterion (IMPLEMENTATION-PLAN §08)

> Four SDKs hitting one gateway; entity graph updates live in a React app.

## Open Decisions to Resolve at Phase 2 Kickoff

| Decision | Status |
|---|---|
| Connect-ES version (`@connectrpc/connect`) | Confirm current stable release before generating |
| buf CLI version + `buf.gen.yaml` config | Confirm before generating |
| protoc-gen-connect-go version | Confirm before generating |
| Grpc.Tools / dotnet NuGet package versions | Confirm before generating |

## Non-Goals for Phase 2

- Swift / Kotlin / Dart FFI (Phase 3)
- CRDT engine (Phase 3)
- WebRTC / media (Phase 4)
- Agent protocols (Phase 5)
