# Flint Realtime Fabric — Implementation Plan

> **RFC-FRF-002 · Prometheus AGS**
> Contract-first, feature-based clean architecture in Rust. Nine SDKs collapsed
> into three maintenance patterns. Eight phases, each independently shippable.

| | |
|---|---|
| **Core** | Rust workspace (hexagonal, feature-gated) |
| **Contract** | protobuf + tonic gRPC |
| **SDKs** | native / generated / FFI |
| **Identity** | Ory Kratos + Oathkeeper |
| **AuthZ** | Keto (Zanzibar) + Cedar |
| **CRDT** | Loro \| automerge + redb / SurrealDB |
| **Status** | plan drafted · contract-freeze is the first gate · 2026-06-17 |

Companion document: `docs/flint-realtime-fabric-architecture` (RFC-FRF-001) — the
plane-separation design this plan implements.

---

## 00 · Framing & Scope

The technical scope — Postgres CDC, WebRTC, CRDT, pub/sub, AG-UI / A2A / A2UI,
Matrix, ATProto, BossFang — is large, but each item is a bounded adapter behind a
port. The genuine program risk is the **nine-language SDK surface**. Hand-writing
nine clients is what kills small teams. The entire plan is organized so that
surface collapses into three repeatable patterns.

> **Operating principle.** Freeze the protobuf contract first. Hand-write only
> Rust. Generate Go / C# / browser-TS from the contract. Bind Swift / Kotlin /
> Dart to the **same Rust core** over FFI — which also delivers identical offline
> CRDT merge to every device for free.

---

## 01 · Repository Strategy

A single Cargo workspace is the source of truth: domain, ports, adapters,
gateway, the proto contract, and the Rust-core SDK bindings. Per-language
packages are generated or bound inside it and published to their native
registries by CI — not maintained as separate hand-edited repos.

```
flint-realtime-fabric/            # Cargo workspace, source of truth
├── proto/                        # THE CONTRACT — freeze before SDK gen
│   └── flint/v1/{envelope,entity,agent,signal,sync,authz}.proto
├── crates/
│   ├── frf-domain/               # pure types, serde only, zero infra deps
│   ├── frf-ports/                # trait seams: LogBroker, AuthzProvider, ...
│   ├── frf-app/                  # use-cases: subscribe, publish, RLS pipeline
│   ├── frf-proto/                # tonic-generated from proto/
│   │   # ── infrastructure adapters (one crate each, impl a port) ──
│   ├── frf-broker-iggy/          # LogBroker        -> Apache Iggy (GQAdonis fork)
│   ├── frf-authz-keto/           # AuthzProvider    -> Ory Keto (Zanzibar)
│   ├── frf-identity-ory/         # IdentityVerifier -> Kratos/Oathkeeper JWT
│   ├── frf-policy-cedar/         # action policy    -> Cedar (existing PAUX-1)
│   ├── frf-postgres-cdc/         # WAL logical replication -> spine
│   ├── frf-crdt/                 # Loro/automerge engine + CrdtStore
│   ├── frf-store-surreal/        # server persistence (checkpoints, presence)
│   ├── frf-store-redb/           # on-device durable op-log + snapshots
│   ├── frf-media-str0m/          # sovereign SFU signaling adapter
│   ├── frf-media-livekit/        # hosted conferencing signaling adapter
│   ├── frf-bridge-matrix/        # Tuwunel projection
│   ├── frf-bridge-atproto/       # Tranquil firehose projection + AppView feed
│   ├── frf-agentproto/           # AG-UI / A2A / A2UI schemas + ContentBlock
│   ├── frf-librefang/            # ractor publish/consume actors (BossFang)
│   │   # ── interface / deployable ──
│   ├── frf-gateway/              # Axum 0.8.8: WS mux + tonic gRPC + Connect
│   ├── frf-cli/                  # ops + dev CLI
│   │   # ── SDK surface ──
│   ├── frf-sdk-rust/             # idiomatic Rust client (hand-written)
│   ├── frf-ffi/                  # UniFFI scaffold -> Swift, Kotlin, JVM
│   └── frf-wasm/                 # wasm-bindgen core for browser TS
├── sdks/                         # generated/bound, published by CI — not hand-edited
│   ├── go/   ts/   csharp/
│   ├── swift/   kotlin/   dart/  # flutter_rust_bridge over frf-ffi/core
│   └── entity-management/        # thin RealtimeAdapter on the TS SDK
├── dagger/                       # CI pipelines (replaces GitHub Actions)
└── Cargo.toml                    # [workspace] + shared deps + per-plane features
```

**Why monorepo.** Contract-first generation only works if the `.proto` and every
generated/bound SDK move atomically. A monorepo makes a contract change +
regenerate + bind a single commit. CI publishes to crates.io, npm, pub.dev, Maven
Central, NuGet, and CocoaPods/SPM on tagged release. Native package managers see
proper packages; you maintain one tree.

---

## 02 · Clean Architecture

Each realtime behavior is a *feature* expressed as a port (trait) plus one or more
adapters. The dependency rule is absolute: domain knows nothing; application
depends only on domain + ports; infrastructure and interface depend inward and are
selected by Cargo features so a deployment compiles only the planes it runs.

- **Layer 0 · Domain** — `frf-domain`: `EventEnvelope`, `Channel`, `Offset`,
  `Cursor`, `EntityChange`, `AgentEvent`, `SyncOp`, `Presence`, `SignalEnvelope`,
  typed newtype IDs (`#[repr(transparent)]`). Serde only. No tokio, no I/O.
- **Layer 1 · Application** — `frf-ports` defines the seams; `frf-app`
  orchestrates them: subscribe/publish, the RLS filter pipeline, the 16ms
  coalescing window, checkpoint logic. Testable with mock adapters, no network.
- **Layer 2 · Infrastructure + Interface** — every `frf-*` adapter crate
  implements exactly one port. `frf-gateway` wires selected adapters behind
  WS + gRPC. Swapping Iggy for NATS, or Keto for a stub, is a feature flag —
  never a refactor.

### Port seams

```rust
pub trait LogBroker        { /* publish, subscribe, checkpoint */ }
pub trait AuthzProvider    { /* check(subject, relation, object), expand, write_tuple */ }
pub trait IdentityVerifier { /* verify(jwt) -> Subject + claims */ }
pub trait CrdtStore        { /* load/save snapshot, append ops, version vector */ }
pub trait MediaSignaler    { /* offer/answer/ice relay, session lifecycle */ }
pub trait FederationBridge { /* project external stream -> spine, inject back */ }
```

### Rust discipline (enforced in CI)

`thiserror` in libraries, `anyhow` only at binary edges · `#[non_exhaustive]` on
public enums · no `unwrap()/expect()` in library crates (clippy-denied) ·
`clippy::pedantic` + `deny(warnings)` gate · pinned MSRV · sans-I/O cores where
the dependency allows (str0m is sans-I/O) · `tracing` spans across every port
boundary · newtype IDs over bare strings · semver discipline on `frf-domain` and
every SDK.

---

## 03 · Protobuf / gRPC Contract

The proto is the optimization layer the constraints call for and the contract
every non-Rust SDK is generated from. Streaming RPCs for subscribe; unary for
publish, checkpoint, and authz. CRDT ops are already binary (automerge/Loro) —
they ride as `bytes`, never re-encoded.

```protobuf
service FlintRealtime {
  // server-streaming: the subscribe fan-out
  rpc Subscribe(SubscribeRequest) returns (stream EventEnvelope);
  rpc Publish(PublishRequest)     returns (PublishAck);
  rpc Checkpoint(CheckpointReq)    returns (CheckpointAck);
  // bidi: CRDT sync + WebRTC signaling envelopes
  rpc SyncStream(stream SyncFrame) returns (stream SyncFrame);
  rpc Signal(stream SignalFrame)   returns (stream SignalFrame);
}

message EventEnvelope {
  uint32 v = 1; Op op = 2; string type = 3; string id = 4;
  bytes  data = 5;            // JSON or CRDT-binary, opaque
  string ts = 6; string tenant = 7; string origin = 8; string trace = 9;
}
```

**Transport tiers from one contract.** gRPC (HTTP/2) for native + server-to-server
SDKs. Browsers can't speak raw gRPC streaming reliably, so the gateway also serves
the **Connect protocol** (Connect-ES) and the existing Phoenix-style WebSocket mux
for `prometheus-entity-management`. One contract, three wire surfaces — no second
schema.

---

## 04 · SDK Strategy — Nine Languages, Three Patterns

This is the table that makes the program survivable. Only Rust is hand-written.
Everything else is generated from the frozen contract or bound to the shared Rust
core — so business logic, CRDT merge, and reconnection live in exactly one place.

| SDK | Pattern | Mechanism | CRDT core? |
|---|---|---|---|
| Rust | Hand-written | `frf-sdk-rust` — full protocol, gRPC + WS | native |
| Go | Generated | connect-go / tonic-compatible from proto + thin shim | via gateway relay |
| C# | Generated | grpc-dotnet from proto + idiomatic wrapper | via gateway relay |
| TS/JS (browser) | Generated + WASM | Connect-ES; optional `frf-wasm` for in-browser CRDT | optional WASM |
| entity-management | Adapter | thin `RealtimeAdapter` on the TS SDK — not a new SDK | inherits TS |
| Swift (iOS/macOS) | FFI bind | UniFFI over `frf-ffi` / Rust core | shared core |
| Kotlin (Android+JVM) | FFI bind | UniFFI over `frf-ffi` / Rust core | shared core |
| Java (Android) | FFI bind | consumes the UniFFI *Kotlin* binding — no separate hand-write | shared core |
| Dart / Flutter | FFI bind | flutter_rust_bridge over the Rust core | shared core |

> **Honest overlap.** Java-for-Android and Kotlin are not two builds. Kotlin is
> Android's language; the UniFFI-generated Kotlin binding is callable from Java
> directly. Build the Kotlin binding; expose a Java-friendly facade only if a
> pure-JVM-server consumer actually appears.

---

## 05 · Realtime Behavior Map

Each requested behavior is a feature behind a port. None is special
infrastructure — they all publish facts to, or consume facts from, the spine,
with media and live-CRDT bypassing it per the Fabric separation contract.

| Behavior | Crate | On spine? | Notes |
|---|---|---|---|
| Supabase-like Postgres realtime | `frf-postgres-cdc` | yes | Logical replication slot → decode → spine → Keto RLS → fan-out |
| Pub/sub + broadcast | `frf-app` | opt | Durable on spine; ephemeral broadcast may use relay-direct fast path |
| Presence | `frf-app` + store | thin | Heartbeat + state; short-retention spine topic |
| CRDT sync | `frf-crdt` | live: no | P2P over WebRTC; checkpoints + announce on spine |
| WebRTC media / conferencing | `frf-media-*` | never | str0m sovereign; LiveKit hosted; signaling-only on spine |
| AG-UI events | `frf-agentproto` | yes | Agent→UI ContentBlock stream on `flint:agent` |
| A2A (agent-to-agent) | `frf-agentproto` | yes | Inter-agent envelopes; durable + replayable |
| A2UI | `frf-agentproto` | yes | Structured UI-intent events; same channel family |
| Matrix protocol events | `frf-bridge-matrix` | projected | Tuwunel owns room DAG; spine indexes a copy |
| AT Protocol | `frf-bridge-atproto` | projected | Tranquil firehose → spine → custom AppView consumers |
| BossFang / LibreFang | `frf-librefang` | yes | ractor publish + consume actors into app state |

---

## 06 · CRDT & Offline Persistence

The persistence design lets a device work offline, accumulate changes, and
converge cleanly on reconnect. Because the CRDT engine lives in the Rust core,
on-device durability is identical on every native platform.

### Three storage tiers

```
ON-DEVICE   frf-store-redb     pure-Rust embedded op-log + snapshots (all native platforms)
            browser            IndexedDB via frf-wasm
SERVER      frf-store-surreal  CRDT checkpoints, op history, presence, version vectors
SOURCE      Postgres           entity source-of-truth feeding frf-postgres-cdc
```

### Online / offline lifecycle

Offline, a device appends ops to its local redb op-log and applies them
optimistically. On reconnect it sends its version vector; the gateway (or a peer
over WebRTC) computes the delta and exchanges only missing ops — incremental, not
full-state. The spine carries the *checkpoint* and an *announce* so a cold device
knows where to resume. Co-present devices skip the server entirely and reconcile
peer-to-peer, then one of them checkpoints the merged state back.

> **Engine decision — still open.** **Loro** vs **automerge-rs** for structured
> entity state; both are mature Rust CRDTs with binary encodings and incremental
> sync. Loro tends to win on memory/perf and rich-document range ops; automerge
> has the longer-running ecosystem and automerge-repo sync patterns. Decide before
> Phase 3 and commit — this choice is load-bearing for every FFI SDK.
> diamond-types remains text-only.

---

## 07 · Identity, AuthZ & Rights

Four concerns, placed deliberately rather than piled together. Identity proves
who; Keto decides who-can-see-what as a relationship graph; Cedar governs
which-actions-under-what-conditions; rights management is users writing their own
relation tuples.

### The pipeline

```
// authN — at connect
client JWT (Kratos-issued) → Oathkeeper/verify → Subject + claims
// authZ — at subscribe (coarse, cached)
Subscribe(realtime:entity:Order, tenant) → Keto.check(subject, "view", topic) → allow/deny
// RLS — at fan-out (per object)
for each EventEnvelope: Keto.check(subject, "view", object_id) before delivery
// action policy — orthogonal
mutating ops → Cedar.is_authorized(principal, action, resource, context)
```

### Placement: Keto vs Cedar

**Keto (Zanzibar / ReBAC)** owns realtime visibility — "can subject see object" is
exactly relation-tuple checking, with the check-cache and userset expansion
Zanzibar was built for. **Cedar (policy / ABAC)** stays where PAUX-1 already uses
it: action authorization and threshold policy. Don't force one to do the other's
job; they compose at the gateway.

### User-controlled rights management

A resource owner grants or revokes access by writing/deleting Keto relation tuples
on objects they own — delegation is native to Zanzibar. Expose this through a
guarded API (`owner` meta-permission gates who may grant). Revocation is a tuple
delete; the gateway's check-cache invalidates and in-flight subscriptions are
re-evaluated. Genuine user-held control, not an admin console.

> **The real scaling hazard.** Per-event, per-subscriber Keto checks do *not*
> scale to high fan-out naively. Mitigations are mandatory: scope visibility at
> *subscribe* time so topics are pre-filtered by tenant/relation; partition spine
> topics along the relation graph; cache check results keyed by
> `(subject, relation, object)` with tuple-delete invalidation. Design this in
> Phase 1, not after a load test surprises you.

---

## 08 · Phased Build Plan

The ordering protects two things: the proto contract freezes before any generated
SDK exists (or they churn endlessly), and the authz scaling design lands in
Phase 1 (or it's retrofitted painfully). Mobile/FFI waits for the CRDT engine
decision.

### Phase 0 — Foundations & contract freeze
Workspace, `frf-domain` + `frf-ports`, the `proto/` contract, tonic codegen,
Dagger CI with clippy-pedantic / MSRV / deny-warnings gates.
- **Exit:** compiling skeleton; proto v1 tagged frozen; CI green
- **Crates:** domain, ports, proto, gateway-stub

### Phase 1 — Spine + Postgres CDC + RLS + Rust SDK
IggyBroker, Axum gateway (WS+gRPC), `frf-postgres-cdc` logical replication, Ory
identity + Keto check pipeline with caching, `frf-sdk-rust`.
- **Exit:** Supabase-like entity sync end-to-end with RLS; Rust client; cache invalidation working
- **Crates:** broker-iggy, identity-ory, authz-keto, postgres-cdc, gateway, sdk-rust

### Phase 2 — Generated SDKs + entity-management adapter
Go, C#, browser-TS (Connect-ES) generated from frozen proto; the
`prometheus-entity-management` RealtimeAdapter on the TS SDK.
- **Exit:** four SDKs hitting one gateway; entity graph updates live in a React app
- **SDKs:** go, csharp, ts, entity-management

### Phase 3 — CRDT core + offline persistence + FFI tier
Engine decision (Loro/automerge) committed; `frf-crdt`, `frf-store-redb`; UniFFI
(Swift, Kotlin) + flutter_rust_bridge (Dart) over the core; offline op-log +
incremental reconnect.
- **Exit:** mobile app edits offline, reconnects, converges; identical merge on all three platforms
- **Crates/SDKs:** crdt, store-redb, ffi; swift, kotlin, dart

### Phase 4 — WebRTC media + signaling
str0m sovereign SFU + LiveKit hosted, both signaling over the spine; AI-voice
session wiring (OpenAI Realtime on the media plane, agent events on the spine).
- **Exit:** P2P + SFU calls; signaling auditable as spine facts; media never on the log
- **Crates:** media-str0m, media-livekit

### Phase 5 — Agent protocols + BossFang
`frf-agentproto` for AG-UI / A2A / A2UI event schemas + ContentBlock emission;
`frf-librefang` ractor publish/consume actors.
- **Exit:** agent events stream to UI via SDKs; LibreFang publishes + consumes spine facts
- **Crates:** agentproto, librefang

### Phase 6 — Federation bridges
Matrix (Tuwunel) projection; ATProto (Tranquil) firehose → spine → custom AppView
consumers with durable offsets. Last because they integrate systems you already run.
- **Exit:** Matrix room + ATProto records surface through the unified socket; AppView serves queries
- **Crates:** bridge-matrix, bridge-atproto

### Phase 7 — Hardening & release automation
AuthZ fan-out load test + cache tuning, user-controlled rights API, observability
(tracing/metrics), release automation publishing every SDK to its registry on tag.
- **Exit:** load targets met; rights delegation live; one tag → all registries
- **Crates:** all; dagger release pipelines

---

## 09 · Risks & Decisions

| Item | Position |
|---|---|
| Nine-SDK surface (dominant risk) | Collapsed to three patterns; only Rust hand-written. Every hand-written SDK is a permanent tax. |
| Proto stability | Freeze v1 in Phase 0. SDKs built before freeze churn. Breaking changes are a new proto version, not an edit. |
| CRDT engine | **OPEN.** Loro vs automerge-rs — decide before Phase 3; propagates into every FFI binding. |
| AuthZ at fan-out | Per-event Keto checks don't scale naively. Subscribe-time scoping + topic partitioning + check-cache in Phase 1. |
| Iggy maturity (pre-1.0) | Accepted. LogBroker trait keeps NATS/Redpanda a swap; Sync-Mesh P2P is the availability fallback. |
| Tooling currency | Confirm current versions + language coverage of UniFFI, flutter_rust_bridge, Connect, tonic at Phase 0 kickoff. |
| Java vs Kotlin | One UniFFI Kotlin binding serves both. Don't double-build. |
| Team scale | Multi-quarter for a small team. Phases sequenced so each is independently useful — ship and harvest before advancing. |
