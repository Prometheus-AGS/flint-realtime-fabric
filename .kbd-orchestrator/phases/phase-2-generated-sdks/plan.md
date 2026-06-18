# Plan: Phase 2 — Generated SDKs + entity-management adapter

> RFC-FRF-002 · Prometheus AGS
> Planned: 2026-06-18
> Backend: OpenSpec
> Phase: phase-2-generated-sdks

---

## Ordering Rationale

Changes are ordered by dependency. The security fix (p2-c001) must land first
because Phase 2 publish-path SDK clients would otherwise bypass write
authorization. Proto annotation fixes (p2-c002) and codegen config (p2-c003)
must precede SDK generation. Gateway tonic registration (p2-c007) must be
complete before any SDK smoke test (p2-c010) runs. The entity-management
adapter and admin-ui (p2-c008, p2-c009) depend on the TS SDK (p2-c005) and
the gateway tonic service (p2-c007) respectively.

---

## Change List (10 changes)

| # | ID | Title | Depends on | Agent |
|---|---|---|---|---|
| 1 | p2-c001-publish-authz-fix | Add Keto write-authz to `PublishUseCase` | — | claude-code |
| 2 | p2-c002-proto-csharp-namespace | Add `csharp_namespace` option to all 6 proto files | — | claude-code |
| 3 | p2-c003-buf-config | Create `buf.yaml` + `buf.gen.yaml`; install codegen plugins | p2-c002 | claude-code |
| 4 | p2-c004-sdk-go | Generate Go SDK; `sdks/go/go.mod`; client smoke wrapper | p2-c003 | claude-code |
| 5 | p2-c005-sdk-ts | Generate TS SDK via buf; `sdks/ts/package.json`; export Connect client | p2-c003 | claude-code |
| 6 | p2-c006-sdk-csharp | C# SDK via Grpc.Tools MSBuild; `sdks/csharp/FlintSdk.csproj` | p2-c002 | claude-code |
| 7 | p2-c007-gateway-tonic-service | Register tonic `SpineService` + `EntityService` in `frf-gateway` | p2-c001 | claude-code |
| 8 | p2-c008-entity-management-adapter | `sdks/entity-management/` RealtimeAdapter on TS SDK | p2-c005 | claude-code |
| 9 | p2-c009-admin-ui-scaffold | `admin-ui/` React 19 + Vite 7; entity-management demo | p2-c008 | claude-code |
| 10 | p2-c010-e2e-smoke | E2E: all 4 SDKs subscribe + publish via gateway; assert receipt | p2-c004, p2-c005, p2-c006, p2-c007, p2-c009 | claude-code |

---

## Per-Change Specifications

### p2-c001 — Add Keto write-authz to `PublishUseCase`

**Crates:** `crates/frf-app`

**What it does:**
Add `AuthzProvider` to `PublishUseCase<L, A, I>` (currently `<L, I>`). After
JWT verification, check `authz.check(RelationTuple { relation: "publish", … })`
before calling `broker.publish()`. Mirror the pattern used in `SubscribePipeline`.

**Files changed:**
- `crates/frf-app/src/publish.rs` — add `authz: Arc<A>`, `A: AuthzProvider` bound, Keto check
- `crates/frf-app/src/lib.rs` — re-export updated `PublishUseCase`
- `crates/frf-gateway/src/lib.rs` — update `AppState<L,A,I>` wiring: pass `authz` to `PublishUseCase::new()`
- `crates/frf-gateway/src/routes/publish.rs` — no change (uses `state.publish_usecase.execute()`)
- `crates/frf-gateway/src/main.rs` — pass `Arc::clone(&authz)` to `PublishUseCase::new()`
- Tests: update the 2 `PublishUseCase` unit tests to pass a mock `AuthzProvider`; add 1 test for `Forbidden` on denied publish

**Exit:** `cargo test -p frf-app` + `cargo check --workspace` green; no `#[allow]` suppressors.

---

### p2-c002 — Add `csharp_namespace` to all 6 proto files

**Files:** `proto/flint/v1/{envelope,entity,agent,signal,sync,authz}.proto`

**What it does:**
Add `option csharp_namespace = "PrometheusAgs.Frf.Flint.V1";` after the
`go_package` option in each file. Without this the C# generator uses the
proto package name which contains dots (invalid as C# namespace components).

**Exit:** `buf lint` passes; `grep csharp_namespace proto/flint/v1/*.proto` shows 6 hits.

---

### p2-c003 — buf config + codegen plugins

**Files (new):**
- `proto/buf.yaml` — workspace, lint, breaking config (see assessment §6)
- `proto/buf.gen.yaml` — Go + TS plugin config (see assessment §6)

**Install steps (run once, documented in tasks.md):**
```bash
brew upgrade buf   # 1.65.0 → 1.71.0
go install connectrpc.com/connect/cmd/protoc-gen-connect-go@v1.20.0
```

**TS plugins via buf remote** (no local install needed for `buf generate`):
`buf.build/bufbuild/es@v2.12.0` + `buf.build/connectrpc/es@v1.7.0`

**What it does:**
- Creates `proto/buf.yaml` with `buf.build/prometheusags/frf` BSR module name
- Creates `proto/buf.gen.yaml` pointing Go output to `../sdks/go/gen` and TS output to `../sdks/ts/src/gen`
- Runs `buf generate` from `proto/` — creates generated files in both output paths
- Verifies `buf lint` passes on the frozen proto contract

**Exit:** `buf lint` ✅; generated files exist at `sdks/go/gen/` and `sdks/ts/src/gen/`; `buf breaking --against '.git#tag=proto-v1'` confirms no breaking changes.

---

### p2-c004 — Go SDK

**Directory:** `sdks/go/`

**What it does:**
- Creates `sdks/go/go.mod` with module `github.com/prometheusags/frf/sdks/go`; deps: `connectrpc.com/connect@v1.20.0`, `google.golang.org/protobuf@v1.36.11`
- Runs `go mod tidy`
- Creates `sdks/go/client/spine_client.go` — thin `SpineClient` struct wrapping the generated Connect stub for `SpineService` (Publish, Subscribe, Ack)
- Creates `sdks/go/README.md` documenting usage
- Creates `sdks/go/client/smoke_test.go` (`//go:build integration`) — tests `Publish` + `Subscribe` against a local gateway; skipped in normal `go test ./...`

**Exit:** `go build ./...` from `sdks/go/` exits 0; `go vet ./...` clean; integration test file compiles.

---

### p2-c005 — TS SDK

**Directory:** `sdks/ts/`

**What it does:**
- Creates `sdks/ts/package.json` with name `@prometheusags/frf-ts-sdk`; deps: `@connectrpc/connect@2.1.2`, `@connectrpc/connect-web@2.1.2`, `@bufbuild/protobuf@2.12.0`; devDeps: `typescript`, `tsup`
- Creates `sdks/ts/tsconfig.json`
- Creates `sdks/ts/src/index.ts` — re-exports generated types + a `createSpineClient(baseUrl)` factory using `@connectrpc/connect-web` transport
- Creates `sdks/ts/src/client.ts` — typed `SpineClient` class wrapping Subscribe (returns `AsyncIterable<EventEnvelope>`) and Publish
- Runs `pnpm install` + `pnpm build` (tsup)

**Exit:** `pnpm typecheck` exits 0; `pnpm build` produces `dist/`; no `any` types.

---

### p2-c006 — C# SDK

**Directory:** `sdks/csharp/`

**What it does:**
- Creates `sdks/csharp/FlintSdk.csproj` with `<PackageReference Include="Grpc.AspNetCore" Version="2.80.0" />`, `Google.Protobuf@3.35.1`, `Grpc.Tools@2.81.1`
- Creates `sdks/csharp/Protos/` — copies all 6 proto files (Grpc.Tools generates C# from local proto copies)
- Creates `sdks/csharp/SpineClient.cs` — thin `SpineClient` wrapper over generated `SpineService.SpineServiceClient`
- Runs `dotnet build`
- Creates `sdks/csharp/SmokeTest.cs` (xunit, `[Trait("Category","Integration")]` skip tag) — Publish + Subscribe round-trip

**Exit:** `dotnet build sdks/csharp/` exits 0; `dotnet test --filter Category!=Integration` passes.

---

### p2-c007 — Gateway tonic/Connect service registration

**Crates:** `crates/frf-gateway`, `crates/frf-app`, `crates/frf-proto`

**What it does:**
Implement the tonic `SpineService` and `EntityService` server traits in `frf-gateway`
and compose them with the existing Axum router using the `axum::Router` + tonic
`Routes` tower layer pattern.

**New files:**
- `crates/frf-gateway/src/grpc/spine.rs` — `SpineServiceImpl<L,A,I>` implementing tonic `SpineService`:
  - `publish`: delegates to `PublishUseCase`; extracts Bearer from metadata
  - `subscribe`: delegates to `SubscribePipeline`; streams `EventEnvelope` protobuf messages
  - `ack`: not yet implemented (returns `Unimplemented`)
- `crates/frf-gateway/src/grpc/entity.rs` — `EntityServiceImpl` stub (returns `Unimplemented` for now)
- `crates/frf-gateway/src/grpc/mod.rs`

**Modified files:**
- `crates/frf-gateway/Cargo.toml` — add `frf-proto` dep; add `tonic::transport` features
- `crates/frf-gateway/src/lib.rs` — export `grpc` module; update `build_router` to merge tonic routes
- `crates/frf-gateway/src/main.rs` — add tonic service to the combined router

**Exit:** `cargo check --workspace` green; `cargo test -p frf-gateway` passes; gateway can be started and `grpcurl` (or equivalent) can hit `/flint.v1.SpineService/Publish`.

---

### p2-c008 — entity-management RealtimeAdapter

**Directory:** `sdks/entity-management/`

**What it does:**
Thin TypeScript adapter that bridges `prometheus-entity-management` entity
state to the realtime fabric via the TS SDK.

- `package.json` with name `@prometheusags/entity-management-realtime`; peer dep on `@prometheusags/frf-ts-sdk`
- `src/RealtimeAdapter.ts` — class `RealtimeAdapter`:
  - `constructor(gatewayUrl: string, bearerToken: string)`
  - `connect(channelPath: string): void` — opens a Subscribe stream; merges `EventEnvelope` payloads into local entity state via `Map<entityId, EntityChange>`
  - `disconnect(): void`
  - `getEntity(id: string): EntityChange | undefined`
  - `onEntityChange(cb: (change: EntityChange) => void): Unsubscribe`
- `src/hooks/useEntity.ts` — React hook: `useEntity(adapter, entityId)` returns `EntityChange | undefined`; re-renders on change
- `src/index.ts` — re-exports
- `tsconfig.json`, `tsup.config.ts`
- Unit tests for `RealtimeAdapter` using `vitest` + mock stream

**Exit:** `pnpm build` ✅; `pnpm test` ✅ (unit tests, no live gateway needed); no `any` types.

---

### p2-c009 — admin-ui scaffold

**Directory:** `admin-ui/`

**What it does:**
Scaffold the React 19 + Vite 7 admin app as described in CLAUDE.md and wire
the entity-management adapter to demonstrate a live entity graph.

- `admin-ui/package.json` — React 19, Vite 7, shadcn-ui, Base UI (latest), TypeScript
- `admin-ui/vite.config.ts`
- `admin-ui/tsconfig.json`
- `admin-ui/index.html`
- `admin-ui/src/main.tsx`
- `admin-ui/src/App.tsx` — wraps the entity demo in a provider
- `admin-ui/src/features/entities/` — feature folder:
  - `components/EntityGraph.tsx` — live entity list subscribing to the realtime fabric
  - `hooks/useEntitySubscription.ts` — wires `RealtimeAdapter` to React state
  - `types.ts`
  - `pages/EntitiesPage.tsx`
- `admin-ui/src/core/` — routing, auth shell skeleton
- `admin-ui/src/infrastructure/` — gateway URL config from env var

Runs `pnpm install` + `pnpm build` to verify the production bundle.

**Exit:** `pnpm typecheck` ✅; `pnpm build` ✅; admin-ui dev server starts and renders the entity graph page (verified visually or via snapshot test).

---

### p2-c010 — E2E smoke test

**What it does:**
End-to-end smoke test asserting all four SDKs can subscribe and receive a
published event from the running gateway.

- `tests/e2e/` directory at workspace root
- `tests/e2e/smoke_test.sh` — orchestrates: start gateway, publish via `curl` (HTTP), assert receipt via Go + TS + C# test binaries
- `tests/e2e/go/main.go` — Go subscriber: `SpineClient.Subscribe`, wait for event, print and exit 0
- `tests/e2e/ts/smoke.ts` — TS subscriber: `SpineClient.subscribe`, wait for event, exit 0
- `tests/e2e/csharp/Smoke.cs` — C# subscriber: subscribe, wait, exit 0
- `tests/e2e/README.md` — how to run

All E2E tests are marked as `#[ignore]` / skipped by default in CI (require live gateway).

**Exit:** `./tests/e2e/smoke_test.sh` exits 0 against a local gateway; each SDK binary exits 0.

---

## Implementation Notes

### tonic + Axum hybrid routing (p2-c007)

```rust
// Pattern: merge tonic Routes into Axum Router via tower
let grpc_router = tonic::transport::Server::builder()
    .add_service(SpineServiceServer::new(spine_impl))
    .add_service(EntityServiceServer::new(entity_impl))
    .into_router();

let app = axum_router
    .merge(grpc_router);
```

### C# proto copies vs buf (p2-c006)

Grpc.Tools generates C# by finding `*.proto` files under the project directory
via MSBuild `<Protobuf Include="Protos/**/*.proto" />`. Proto files must be
physically present — they cannot be pointed at a parent directory without
custom MSBuild. Copy strategy: `sdks/csharp/Protos/` mirrors `proto/flint/v1/`
exactly; a CI script asserts they stay in sync.

### entity-management adapter state model (p2-c008)

`RealtimeAdapter` maintains an in-memory `Map<string, EntityChange>` and
re-emits via callbacks. Subscribers call `onEntityChange` for push updates.
`useEntity` wraps this in a `useEffect` + `useState` pair. No external state
manager (Zustand/Jotai) in the adapter itself — callers wire it as needed.

---

## Toolchain Setup Script

Included in p2-c003 tasks:

```bash
#!/usr/bin/env bash
set -euo pipefail
brew upgrade buf
go install connectrpc.com/connect/cmd/protoc-gen-connect-go@v1.20.0
echo "Codegen toolchain ready."
```
