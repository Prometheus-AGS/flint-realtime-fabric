# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Mandatory Reading Before Any Code Generation

Read **`docs/IMPLEMENTATION-PLAN.md`** (RFC-FRF-002) in full before generating any code. Follow the phase order exactly. Do not skip the contract-freeze gate. **Halt at each phase boundary for explicit approval — do not proceed to the next phase or produce artifacts belonging to a future phase without it.**

Base rules for all agents are in **`docs/PROMETHEUS-BASE-RULES.md`** (Rules 1–40). This CLAUDE.md adds project-specific constraints that supplement them.

---

## File Size Limit

**No file over 500 lines.** When a file approaches 500 lines, create a directory module and split it into architecturally sound sub-modules. This applies to every language (Rust, TypeScript, proto files, config files).

---

## Current Repository State

The root is a fresh `cargo init` stub. **Before any Phase 1 work**, Phase 0 must complete:

1. Replace root `Cargo.toml` with a `[workspace]` manifest (see Phase 0 in `docs/IMPLEMENTATION-PLAN.md`).
2. Delete `src/main.rs` — the root is a workspace, not a binary.
3. Create `crates/` directory structure with `frf-domain`, `frf-ports`, `frf-proto`, `frf-gateway`.
4. Author `proto/flint/v1/*.proto` and tag `proto-v1`.
5. Set up `dagger/` CI pipelines.

**Phase 0 exit criteria:** workspace compiles; `proto-v1` tagged; CI green; `frf-gateway` serves `/healthz`. Then halt for approval.

---

## Workspace Structure (Target)

```
flint-realtime-fabric/
├── proto/flint/v1/          # THE CONTRACT — freeze before SDK generation
├── crates/
│   ├── frf-domain/          # Layer 0: pure types, serde only, zero infra deps
│   ├── frf-ports/           # Layer 1: trait seams (no implementations)
│   ├── frf-app/             # Layer 1: use-cases against ports only
│   ├── frf-proto/           # generated from proto/ via tonic-build
│   ├── frf-broker-iggy/     # Adapter: LogBroker → Apache Iggy
│   ├── frf-authz-keto/      # Adapter: AuthzProvider → Ory Keto
│   ├── frf-identity-ory/    # Adapter: IdentityVerifier → Kratos/Oathkeeper
│   ├── frf-policy-cedar/    # Adapter: action policy → Cedar
│   ├── frf-postgres-cdc/    # Adapter: WAL logical replication → spine
│   ├── frf-crdt/            # Adapter: Loro/automerge + CrdtStore
│   ├── frf-store-surreal/   # Adapter: server persistence (SurrealDB 3.x)
│   ├── frf-store-redb/      # Adapter: on-device op-log (redb)
│   ├── frf-media-str0m/     # Adapter: sovereign SFU signaling
│   ├── frf-media-livekit/   # Adapter: hosted conferencing signaling
│   ├── frf-bridge-matrix/   # Adapter: Tuwunel projection
│   ├── frf-bridge-atproto/  # Adapter: Tranquil firehose projection
│   ├── frf-agentproto/      # AG-UI / A2A / A2UI schemas + ContentBlock
│   ├── frf-librefang/       # ractor publish/consume actors (BossFang)
│   ├── frf-gateway/         # Interface: Axum 0.8.8, WS mux + gRPC + Connect
│   ├── frf-cli/             # Interface: ops + dev CLI
│   ├── frf-sdk-rust/        # SDK: hand-written Rust client
│   ├── frf-ffi/             # SDK: UniFFI scaffold → Swift, Kotlin
│   └── frf-wasm/            # SDK: wasm-bindgen → browser TS
├── sdks/                    # generated/bound — not hand-edited
│   ├── go/  ts/  csharp/
│   ├── swift/  kotlin/  dart/
│   └── entity-management/   # thin RealtimeAdapter on TS SDK
├── admin-ui/                # React 19 / Vite 7 / shadcn / Base UI admin app
│   └── src/features/        # feature-based clean architecture
├── dagger/                  # CI pipelines
└── Cargo.toml               # [workspace] manifest
```

---

## Architecture: The Absolute Dependency Rule

```
Domain (frf-domain)
  ↑ imported by
Application (frf-app, frf-ports)
  ↑ imported by
Infrastructure adapters (frf-broker-*, frf-authz-*, ...)
  ↑ wired by
Interface (frf-gateway)
```

**Nothing in `frf-domain` or `frf-app` may import an adapter crate.** The compiler must make violations impossible — keep adapter crates out of `frf-domain` and `frf-app` `[dependencies]`. Every adapter implements exactly one port. Composition happens in `frf-gateway` only.

---

## Stack (Confirm Currency at Phase 0 Kickoff)

| Concern | Choice |
|---|---|
| Web / gateway | Axum 0.8.8 |
| gRPC | tonic + prost |
| Actors | ractor (BossFang / LibreFang) |
| Event spine | Apache Iggy (GQAdonis fork) behind `LogBroker` |
| Identity | Ory Kratos / Oathkeeper (JWT) |
| AuthZ | Ory Keto (Zanzibar) + Cedar (PAUX-1) |
| CRDT | Loro **or** automerge-rs — **OPEN, decide before Phase 3** |
| On-device store | redb |
| Server store | SurrealDB 3.x |
| Postgres | PostgreSQL 17 (CDC via logical replication slot) |
| Media SFU | str0m (sovereign) / LiveKit (hosted) |
| Federation | Tuwunel (Matrix), Tranquil (ATProto) |
| FFI | UniFFI (Swift, Kotlin) + flutter_rust_bridge (Dart) |
| Browser transport | Connect-ES + WS mux |
| CI | Dagger |
| Admin UI | React 19 + Vite 7 + shadcn-ui + Base UI (latest) |

> **Versions for UniFFI, flutter_rust_bridge, Connect, tonic shift.** Confirm current releases and language coverage before committing the FFI/codegen approach. Use Tavily or Firecrawl to validate.

---

## Rust Code Quality Gates (All Must Pass in CI)

These are hard quality gates — a violation is a failure, not a style nit:

```toml
# In every library crate's Cargo.toml or workspace:
[lints.clippy]
pedantic = "warn"

[profile.dev]
# deny(warnings) enforced in CI via RUSTFLAGS="-D warnings"
```

- **No `unwrap()` / `expect()` in library crates.** Use `thiserror` for library errors; `anyhow` only at binary edges (`frf-gateway`, `frf-cli`). Deny via `clippy::unwrap_used`.
- **`clippy::pedantic` + `deny(warnings)`** must pass on every commit.
- **`#[non_exhaustive]`** on all public enums.
- **Newtype IDs** with `#[repr(transparent)]` over bare `String` for all entity IDs.
- **`tracing` spans** across every port boundary call.
- **MSRV** must be pinned in the workspace manifest.
- **Semver discipline** on `frf-domain` and every SDK crate.

---

## Common Commands

```bash
# Check compilation (fast — no linking)
cargo check --workspace

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p frf-domain

# Run a single test
cargo test -p frf-domain -- serialization::test_event_envelope_roundtrip

# Clippy (CI-equivalent)
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

# Format check (CI)
cargo fmt --check --all

# Format (apply)
cargo fmt --all

# Build release
cargo build --workspace --release

# Build proto codegen (inside frf-proto)
cargo build -p frf-proto

# Run gateway
cargo run -p frf-gateway

# Admin UI (once admin-ui/ is scaffolded)
cd admin-ui && pnpm install
cd admin-ui && pnpm dev
cd admin-ui && pnpm build
cd admin-ui && pnpm typecheck
```

---

## Protobuf Contract Rules

The proto is the source of truth for every non-Rust SDK.

1. **Freeze first.** Tag `proto-v1` before generating or hand-writing any SDK. SDKs built before the freeze churn endlessly.
2. **Breaking changes** are a new proto version — never edit `flint/v1/*.proto` after the freeze.
3. `frf-proto` uses `tonic-build` in `build.rs`. The generated code is committed only if CI cannot run `protoc`.
4. Streaming RPCs for subscribe; unary for publish, checkpoint, authz; bidi for CRDT sync and WebRTC signaling.

Proto files live at `proto/flint/v1/{envelope,entity,agent,signal,sync,authz}.proto`.

---

## One Port Per Adapter Rule

Each `frf-*` adapter crate implements **exactly one** port trait. An adapter must not:
- Implement two ports.
- Reach across to another adapter crate directly.
- Import `frf-app` or `frf-domain`-adjacent adapters.

Composition of adapters happens exclusively in `frf-gateway` via Cargo features. A deployment compiles only the planes it runs.

---

## Admin UI Architecture (React 19 / Vite 7)

The admin UI lives in `admin-ui/` and is served by `frf-gateway` via an Axum static embedded-files route (using `rust-embed` or `axum-static`).

**Feature-based structure:**

```
admin-ui/src/
├── features/
│   ├── <domain>/
│   │   ├── components/   # pure rendering, no data fetching
│   │   ├── hooks/        # connect UI to stores, no direct API calls
│   │   ├── stores/       # Zustand stores — call services
│   │   ├── services/     # API calls, SDK calls
│   │   ├── types/
│   │   └── pages/
├── shared/               # genuinely reusable across features
├── core/                 # routing, auth shell, layout
└── infrastructure/       # API client setup, SDK config
```

**Component rules:**
- Components render; hooks coordinate; stores own state; services call APIs.
- No component fetches data directly.
- No `any` types in TypeScript.
- Use shadcn-ui and Base UI (latest) for primitives.
- TSX over JSX everywhere.

---

## SDK Strategy (Do Not Hand-Write Non-Rust SDKs)

| SDK | Pattern | Do Not |
|---|---|---|
| Rust | Hand-written (`frf-sdk-rust`) | — |
| Go, C#, browser-TS | Generated from frozen proto | hand-write |
| Swift, Kotlin, Java | UniFFI over `frf-ffi` | hand-write; Java consumes Kotlin binding |
| Dart / Flutter | flutter_rust_bridge over Rust core | hand-write |
| entity-management | Thin `RealtimeAdapter` on TS SDK | treat as a new SDK |

Only Rust is hand-written. Everything else is generated or FFI-bound. Business logic, CRDT merge, and reconnection logic live in exactly one place.

---

## Naming Conventions

- **Crate directories:** kebab-case (`frf-broker-iggy`)
- **Rust module files:** snake_case (language constraint: `broker_iggy.rs`)
- **TypeScript components:** PascalCase (`UserTable.tsx`)
- **TypeScript hooks:** `use` prefix (`useEntitySync.ts`)
- **Proto files:** snake_case (`envelope.proto`)
- **TSX over JSX** in all generated or hand-written TypeScript UI

---

## Open Decisions (Must Be Resolved Before Advancing Phases)

| Decision | Must Be Made Before |
|---|---|
| CRDT engine: Loro vs automerge-rs | Phase 3 |
| UniFFI / flutter_rust_bridge version | Phase 3 kickoff |
| Connect-ES version | Phase 2 kickoff |
| Tonic version | Phase 0 kickoff |

These are load-bearing choices. Do not code around them without a decision. Surface them explicitly to the operator before proceeding.

---

## Security Constraints

- **Tenant isolation** is enforced at the Keto (Zanzibar) layer, not in application code.
- **Per-event RLS:** Keto `check(subject, "view", object_id)` before every fan-out delivery — design caching at subscribe time to avoid per-event Keto latency at scale.
- **JWT verification** via Oathkeeper at the gateway boundary. Never trust unverified claims downstream.
- **Cedar** governs action policy (mutating ops), not visibility. Do not conflate with Keto.
- **Never log** JWT payloads, relation tuples, or tenant identifiers in debug output.

---

## Phase Gate Protocol

Each phase has an explicit exit criterion in `docs/IMPLEMENTATION-PLAN.md`. The protocol is:

1. Complete all tasks in the current phase.
2. Verify the exit criteria are met.
3. **Stop and report to the operator.**
4. Do not begin the next phase until explicit approval is given.

This is not optional. Do not auto-advance.
