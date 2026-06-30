# Flint Realtime Fabric

A unified, sovereign realtime substrate in Rust: one durable event spine (Apache
Iggy), one multiplexed client socket, and separate transports for media, peer
CRDT sync, and federation. This repository is the contract-first monorepo for the
core, the gateway, and every SDK.

> **For Claude Code and all agent systems:** Read `CLAUDE.md` and
> `docs/PROMETHEUS-BASE-RULES.md` (Rules 1–40) before generating any code.
> Read `docs/IMPLEMENTATION-PLAN.md` (RFC-FRF-002) in full before generating
> any code. Follow the phase order exactly. Do not skip the contract-freeze gate.
> Halt at each phase boundary for explicit approval.

---

## Current State

**Phase 12 complete.** The workspace is fully built and operational.

Active phase: `phase-13-live-layer3-e2e-validation` (pending kick-off).

See `.kbd-orchestrator/current-waypoint.json` for the live orchestration state and
`.kbd-orchestrator/phases/` for per-phase plans, assessments, and reflections.

---

## Repository Structure

```
flint-realtime-fabric/
├── proto/flint/v1/          # THE CONTRACT — frozen at proto-v1
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
│   ├── frf-crdt/            # Adapter: Loro CRDT engine + CrdtStore
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
│   ├── frf-ffi/             # SDK: UniFFI scaffold → Swift, Kotlin
│   └── frf-wasm/            # SDK: wasm-bindgen → browser TS (Loro CRDT)
├── sdks/                    # generated/bound — not hand-edited
│   ├── go/  ts/  csharp/
│   ├── swift/  kotlin/  dart/
│   └── entity-management/   # thin RealtimeAdapter on TS SDK
├── admin-ui/                # React 19 / Vite 7 / shadcn-ui / Base UI
│   └── src/features/        # feature-based clean architecture
├── scripts/
│   ├── smoke-cdc.sh         # CDC replication slot smoke test
│   └── bench-regression-check.sh
├── dagger/
│   └── codegen.ts           # 10-stage CI pipeline
├── docs/
│   ├── IMPLEMENTATION-PLAN.md        # RFC-FRF-002 (authoritative build plan)
│   ├── PROMETHEUS-BASE-RULES.md      # Rules 1–40 for all agents
│   └── decisions/                    # Architecture Decision Records
├── openspec/changes/         # OpenSpec change proposals (active + archive)
├── .kbd-orchestrator/        # KBD orchestration state (travels with repo)
└── Cargo.toml                # [workspace] manifest
```

---

## Architecture

Feature-based hexagonal. `frf-domain` holds pure types (serde only, no I/O).
`frf-ports` defines trait seams (`LogBroker`, `AuthzProvider`, `IdentityVerifier`,
`CrdtStore`, `MediaSignaler`, `FederationBridge`). `frf-app` orchestrates
use-cases against ports only. Every `frf-*` adapter crate implements exactly one
port and is selected by Cargo features, so a deployment compiles only the planes
it runs. `frf-gateway` (Axum 0.8.8) is the deployable: WebSocket mux + tonic gRPC
+ Connect-ES.

**The dependency rule is absolute and points inward:**

```
Domain (frf-domain)
  ↑ imported by
Application (frf-app, frf-ports)
  ↑ imported by
Infrastructure adapters (frf-broker-*, frf-authz-*, ...)
  ↑ wired by
Interface (frf-gateway)
```

Nothing in `frf-domain` or `frf-app` may import an adapter crate. The compiler
enforces this by keeping adapter crates out of `frf-domain` and `frf-app`
`[dependencies]`. Composition happens in `frf-gateway` only.

---

## Prometheus Base Rules

All agents (Claude Code, Codex, Gemini CLI, Roo, Cline, Kilo Code, Librefang,
and any Prometheus/UAR-compatible agent) operating in this repository must follow
**[docs/PROMETHEUS-BASE-RULES.md](docs/PROMETHEUS-BASE-RULES.md)** (Rules 1–40).

Key principles:

| Rule | Principle |
|------|-----------|
| 1 | Think before coding — surface tradeoffs first |
| 2 | Simplicity first — minimum code that solves the problem |
| 3 | Surgical changes — touch only what is necessary |
| 4 | Goal-driven execution — define success criteria first |
| 5 | Truth over fluency — never invent APIs or behavior |
| 8 | Minimize irreversible actions — confirm before destructive ops |
| 11 | Architecture before code — understand the system first |
| 12 | Open standards first — MCP, A2A, AG-UI, WASM, OpenAPI |
| 16 | Strict layering is mandatory — domain ← app ← infra ← interface |
| 25 | Human override always exists |
| 30 | Tests are part of completion |
| 33 | Security is not optional |
| 40 | Stop when done |

Project-specific constraints in `CLAUDE.md` add stricter requirements on top of
these base rules (see Rule 26).

---

## Agent Rules (CLAUDE.md / AGENTS.md)

`CLAUDE.md` in this repository extends the base rules with project-specific
constraints:

- **File size limit:** no file over 500 lines
- **Absolute dependency rule:** enforced by Cargo — domain never imports adapters
- **No `unwrap()` / `expect()` in library crates** — `thiserror` + `anyhow` only at binary edges
- **`clippy::pedantic` + `deny(warnings)`** must pass on every commit
- **`#[non_exhaustive]`** on all public enums
- **Newtype IDs** with `#[repr(transparent)]`
- **One port per adapter** — no adapter implements two ports
- **Proto contract is frozen** — `flint/v1/*.proto` is immutable; breaking changes = new version
- **Phase gate protocol** — halt at each phase boundary for explicit operator approval

---

## Common Commands

```bash
# Compilation check (fast)
cargo check --workspace

# Run all tests
cargo test --workspace

# Clippy (CI-equivalent)
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

# Format check (CI)
cargo fmt --check --all

# Format (apply)
cargo fmt --all

# Build release
cargo build --workspace --release

# Run gateway
cargo run -p frf-gateway

# Admin UI
cd admin-ui && pnpm install && pnpm dev

# Dagger CI pipeline
dagger run ts-node dagger/codegen.ts

# Layer 3 E2E (requires Docker host with DinD)
ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts

# CDC smoke test (requires running compose stack)
bash scripts/smoke-cdc.sh
```

---

## Stack

| Concern | Choice |
|---|---|
| Web / gateway | Axum 0.8.8 |
| gRPC | tonic + prost |
| Actors (BossFang) | ractor (LibreFang) |
| Event spine | Apache Iggy (GQAdonis fork) behind `LogBroker` |
| Identity | Ory Kratos / Oathkeeper (JWT) |
| AuthZ | Ory Keto (Zanzibar) + Cedar (PAUX-1) |
| CRDT engine | Loro 1.13.1 (decision: ADR-001) |
| On-device store | redb |
| Server store | SurrealDB 3.x |
| Postgres | PostgreSQL 17 (CDC via logical replication slot) |
| Media SFU | str0m (sovereign) / LiveKit (hosted) |
| Federation | Tuwunel (Matrix), Tranquil (ATProto) |
| FFI bindings | UniFFI (Swift, Kotlin) + flutter_rust_bridge (Dart) |
| Browser transport | Connect-ES + WS mux |
| CI | Dagger (10-stage pipeline) |
| Admin UI | React 19 + Vite 7 + shadcn-ui + Base UI (latest) |

---

## SDK Strategy

Only **Rust** is hand-written. Everything else is generated or FFI-bound.
Business logic, CRDT merge, and reconnection logic live in exactly one place.

| SDK | Pattern |
|---|---|
| Rust | Hand-written (`frf-sdk-rust`) |
| Go, C#, browser-TS | Generated from frozen proto |
| Swift, Kotlin | UniFFI over `frf-ffi` |
| Dart / Flutter | flutter_rust_bridge over Rust core |
| entity-management | Thin `RealtimeAdapter` on TS SDK |

Java-for-Android consumes the UniFFI Kotlin binding — do not hand-write a
separate Java SDK.

---

## Documents

| File | Description |
|---|---|
| `docs/IMPLEMENTATION-PLAN.md` | RFC-FRF-002 — authoritative phase-by-phase build plan |
| `docs/PROMETHEUS-BASE-RULES.md` | Rules 1–40 for all agents |
| `docs/decisions/` | Architecture Decision Records (ADRs) |
| `CLAUDE.md` | Project-specific agent constraints (extends base rules) |
| `.kbd-orchestrator/` | KBD orchestration state — travels with the repo |
| `openspec/` | OpenSpec change proposals and archive |

---

## License

MIT (workspace default). Confirm before first publish.
