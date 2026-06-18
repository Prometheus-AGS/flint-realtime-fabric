# Assessment: Phase 2 — Generated SDKs + entity-management adapter

> RFC-FRF-002 · Prometheus AGS
> Assessed: 2026-06-18
> Phase: phase-2-generated-sdks

---

## 1. Phase Context

Phase 2 generates four SDKs from the frozen `proto-v1` contract and wires
them to the live `frf-gateway`. The exit criterion is:
> *"Four SDKs hitting one gateway; entity graph updates live in a React app."*

One prerequisite carried from Phase 1 must also be resolved: the missing
Keto authz check on the `PublishUseCase` write path.

---

## 2. Toolchain Inventory

### On-machine tools (confirmed present)

| Tool | Version | Status |
|---|---|---|
| buf CLI | 1.65.0 (installed, latest stable 1.71.0) | ⚠️ STALE — update before use |
| protoc | libprotoc 33.4 | ✅ current |
| protoc-gen-go | v1.36.10 | ✅ current (matches google.golang.org/protobuf v1.36.11) |
| Go | 1.24.3 | ✅ current |
| Node.js | v24.16.0 | ✅ current |
| pnpm | 11.5.2 | ✅ current |
| dotnet | 10.0.107 | ✅ current |

### Missing plugins (must be installed before codegen)

| Plugin | Install Command | Latest Version |
|---|---|---|
| `protoc-gen-connect-go` | `go install connectrpc.com/connect/cmd/protoc-gen-connect-go@latest` | v1.20.0 |
| `@bufbuild/protoc-gen-es` | pnpm (see buf.gen.yaml) | 2.12.0 |
| `@connectrpc/protoc-gen-connect-es` | pnpm (see buf.gen.yaml) | 1.7.0 |

### Confirmed latest versions (as of 2026-06-18)

| Package | Version |
|---|---|
| buf CLI | v1.71.0 |
| connectrpc.com/connect (Go module) | v1.20.0 |
| google.golang.org/protobuf (Go) | v1.36.11 |
| @connectrpc/connect | 2.1.2 |
| @connectrpc/connect-web | 2.1.2 |
| @bufbuild/protobuf | 2.12.0 |
| @bufbuild/protoc-gen-es | 2.12.0 |
| @connectrpc/protoc-gen-connect-es | 1.7.0 |
| Grpc.AspNetCore (NuGet) | 2.80.0 |
| Google.Protobuf (NuGet) | 3.35.1 |
| Grpc.Tools (NuGet) | 2.81.1 |

---

## 3. Proto Contract Audit

### Files in `proto/flint/v1/`

| File | Services | Messages/Enums | go_package | csharp_namespace |
|---|---|---|---|---|
| envelope.proto | SpineService (Publish, Subscribe, Ack) | Channel, Offset, Cursor, EventEnvelope, EventKind | ✅ set | ❌ MISSING |
| entity.proto | EntityService (GetEntity, WatchEntity) | EntityChange, ChangeOp | ✅ set | ❌ MISSING |
| agent.proto | AgentService (RunAgent) | AgentEvent, AgentProtocol | ✅ set | ❌ MISSING |
| signal.proto | SignalService (Signal bidi) | SignalEnvelope | ✅ set | ❌ MISSING |
| sync.proto | SyncService (Sync bidi, GetCheckpoint) | SyncOp, SyncCheckpoint | ✅ set | ❌ MISSING |
| authz.proto | AuthzService (Check, WriteRelation, DeleteRelation) | RelationTuple | ✅ set | ❌ MISSING |

**Gap: all proto files are missing `option csharp_namespace`.** Without this
option, the C# generator will use the package name (`flint.v1`) as the
namespace, which produces invalid C# identifiers. Must be added before C#
codegen: `option csharp_namespace = "PrometheusAgs.Frf.Flint.V1";`

**No `buf.yaml` exists** in `proto/` — needed for `buf lint`, `buf breaking`,
and `buf generate`. Must be created.

---

## 4. Gap Analysis by SDK

### 4.1 Go SDK (`sdks/go/`)

| Item | Status |
|---|---|
| `sdks/go/` directory | ❌ does not exist |
| `buf.gen.yaml` with go plugin config | ❌ does not exist |
| `go.mod` for generated module | ❌ does not exist |
| `protoc-gen-connect-go` installed | ❌ not on PATH |
| Generated `.pb.go` files | ❌ not generated |
| Generated `*_connect.pb.go` stubs | ❌ not generated |
| Smoke test connecting to gateway | ❌ not implemented |

**Missing pre-condition:** `go_package` option already set to
`github.com/prometheusags/frf/proto/flint/v1;flintv1` across all proto files ✅ — no change needed.

### 4.2 C# SDK (`sdks/csharp/`)

| Item | Status |
|---|---|
| `sdks/csharp/` directory | ❌ does not exist |
| `csharp_namespace` option in all proto files | ❌ MISSING — blocker |
| `.csproj` with Grpc.Tools + Google.Protobuf | ❌ does not exist |
| Generated C# files | ❌ not generated |
| Smoke test connecting to gateway | ❌ not implemented |

### 4.3 Browser-TS SDK (`sdks/ts/`)

| Item | Status |
|---|---|
| `sdks/ts/` directory | ❌ does not exist |
| `buf.yaml` in `proto/` | ❌ does not exist |
| `buf.gen.yaml` with ES + connect-es plugins | ❌ does not exist |
| `package.json` for generated module | ❌ does not exist |
| Generated `.ts` files | ❌ not generated |
| `@bufbuild/protoc-gen-es` installed | ❌ not installed |
| `@connectrpc/protoc-gen-connect-es` installed | ❌ not installed |
| Smoke test connecting to gateway | ❌ not implemented |

### 4.4 `entity-management` RealtimeAdapter (`sdks/entity-management/`)

| Item | Status |
|---|---|
| `sdks/entity-management/` directory | ❌ does not exist |
| `package.json` for adapter package | ❌ does not exist |
| `RealtimeAdapter` TypeScript class | ❌ not implemented |
| Connection to `SpineService.Subscribe` via TS SDK | ❌ not implemented |
| Entity state management (subscribe + merge) | ❌ not implemented |
| React hook (`useEntity`) | ❌ not implemented |
| `admin-ui/` React app scaffold | ❌ does not exist |
| Integration with entity-management live in React | ❌ not implemented |

**Dependency chain:** entity-management adapter depends on TS SDK; TS SDK depends on proto generation; proto generation depends on `buf.yaml` + plugins.

---

## 5. Security Prerequisite (Phase 1 HIGH debt)

**`PublishUseCase` missing Keto authz check** — `crates/frf-app/src/publish.rs:35–43` verifies the JWT identity but calls `broker.publish()` without checking write permission via `AuthzProvider::check()`. This means any authenticated tenant can publish to any channel without authorization.

Per CLAUDE.md security constraints: *"Per-event RLS: Keto check(subject, 'view', object_id) before every fan-out delivery"* — the publish path needs a symmetric write-permission check.

**Must be resolved as the first change in Phase 2** before SDK clients add publish-path traffic.

`PublishUseCase` signature change needed:
```rust
pub struct PublishUseCase<L, A, I> {
    broker: Arc<L>,
    authz: Arc<A>,    // ← add AuthzProvider
    identity: Arc<I>,
}
// execute(): verify JWT → check Keto write permission → broker.publish()
```

This also requires updating `AppState` in `frf-gateway` to pass `authz` to `PublishUseCase` (it already has `authz` in the state for `SubscribePipeline`).

---

## 6. `buf.yaml` and `buf.gen.yaml` gaps

### `proto/buf.yaml` (does not exist)

Needed for:
- `buf lint` — enforces proto style rules
- `buf breaking` — detects breaking proto changes against `proto-v1` tag
- `buf generate` — runs codegen plugins

Minimum content:
```yaml
version: v2
modules:
  - path: .
name: buf.build/prometheusags/frf
lint:
  use:
    - DEFAULT
breaking:
  use:
    - FILE
```

### `proto/buf.gen.yaml` (does not exist)

Drives all three SDK generations from a single `buf generate` invocation:
```yaml
version: v2
plugins:
  - remote: buf.build/protocolbuffers/go
    out: ../sdks/go/gen
    opt: paths=source_relative
  - remote: buf.build/connectrpc/go
    out: ../sdks/go/gen
    opt: paths=source_relative
  - remote: buf.build/bufbuild/es
    out: ../sdks/ts/src/gen
    opt: target=ts
  - remote: buf.build/connectrpc/es
    out: ../sdks/ts/src/gen
    opt: target=ts
```

**Note on C#:** buf does not have a first-party C# remote plugin that supports Connect protocol. C# generation must use `Grpc.Tools` via `.csproj` MSBuild integration — not `buf generate`. The C# SDK uses gRPC (not Connect) as the transport.

---

## 7. `frf-gateway` Connect protocol gap

The `frf-gateway` currently:
- Serves `/healthz` via Axum HTTP
- Serves `/v1/publish` via Axum HTTP POST
- Serves `/ws/v1/subscribe` via Axum WebSocket

The Phase 2 SDKs use the **Connect protocol** (HTTP/1.1 + HTTP/2 compatible), which requires tonic/Connect routing in the gateway. The gateway's `frf-proto` crate generates prost types but **does not yet register tonic `Service` implementations**.

Gaps:
1. `frf-gateway` does not implement tonic `SpineService` server trait
2. No Connect-compatible router is wired (`tonic` + `axum` hybrid routing)
3. `axum::Router` + `tonic::transport::Server` compose via `tower` — needs setup

This is a prerequisite for SDK smoke tests.

---

## 8. `admin-ui` gap

`admin-ui/` does not exist. Per CLAUDE.md: *"React 19 / Vite 7 / shadcn-ui / Base UI admin app"*.

The Phase 2 exit criterion requires *"entity graph updates live in a React app"* — this needs at minimum a Vite + React scaffold with the entity-management adapter wired to a running gateway.

---

## 9. Summary of Gaps

| Gap | Severity | Blocks |
|---|---|---|
| `PublishUseCase` missing Keto write-authz check | **CRITICAL** (security) | All Phase 2 publish-path clients |
| `csharp_namespace` missing from all proto files | **HIGH** (codegen blocker) | C# SDK generation |
| `buf.yaml` + `buf.gen.yaml` do not exist | **HIGH** | Go + TS SDK generation |
| `protoc-gen-connect-go` not installed | **HIGH** | Go SDK generation |
| `@bufbuild/protoc-gen-es` + `protoc-gen-connect-es` not installed | **HIGH** | TS SDK generation |
| `sdks/{go,ts,csharp,entity-management}/` do not exist | HIGH | All SDKs |
| tonic `SpineService` not registered in gateway | HIGH | SDK smoke tests |
| `admin-ui/` does not exist | HIGH | Phase 2 exit criterion (React app) |
| buf CLI v1.65.0 installed vs v1.71.0 latest | LOW | buf generate (update before use) |

---

## 10. Recommended Change Order for Plan

1. **p2-c001-publish-authz-fix** — add Keto write-permission check to `PublishUseCase`; update `AppState` wiring in gateway; update 7 unit tests
2. **p2-c002-proto-csharp-namespace** — add `csharp_namespace` option to all 6 proto files
3. **p2-c003-buf-config** — create `proto/buf.yaml` and `proto/buf.gen.yaml`; install `protoc-gen-connect-go`; run `buf generate` for Go + TS
4. **p2-c004-sdk-go** — write `sdks/go/go.mod`, run `buf generate` → Go SDK; add `SpineService` client smoke test
5. **p2-c005-sdk-ts** — write `sdks/ts/package.json`, run `buf generate` → TS SDK; export Connect client
6. **p2-c006-sdk-csharp** — write `sdks/csharp/FlintSdk.csproj` with Grpc.Tools; generate C# via MSBuild; smoke test
7. **p2-c007-entity-management-adapter** — implement `RealtimeAdapter` TypeScript class over TS SDK; `useEntity` hook; exports
8. **p2-c008-admin-ui-scaffold** — scaffold `admin-ui/` with React 19 + Vite 7 + shadcn + Base UI; wire entity-management adapter; live entity graph demo
9. **p2-c009-gateway-tonic-service** — register tonic `SpineService` + `EntityService` in gateway; Connect routing via axum + tower hybrid
10. **p2-c010-e2e-smoke** — E2E test: all 4 SDKs subscribe to gateway; publish event via gateway; assert receipt

**Note on ordering:** p2-c009 (gateway Connect routing) could be done before or interleaved with SDK generation — SDKs can be generated without a running gateway; smoke tests require it. The plan should sequence changes so each has a clear compile + test gate.
