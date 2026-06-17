# Flint Realtime Fabric

A unified, sovereign realtime substrate in Rust: one durable event spine (Apache
Iggy), one multiplexed client socket, and separate transports for media, peer
CRDT sync, and federation. This repository is the contract-first monorepo for the
core, the gateway, and every SDK.

> **For Claude Code:** This README is your build brief. Read
> `docs/IMPLEMENTATION-PLAN.md` (RFC-FRF-002) and
> `docs/flint-realtime-fabric-implementation-plan.html` in full before generating
> any code. Follow the phase order exactly. Do not skip the contract-freeze gate.
> Halt at each phase boundary for explicit approval — do not proceed to the next
> phase or produce artifacts belonging to a future phase without it.

---

## Current state

This directory is a fresh `cargo init` and must be restructured into a workspace
before Phase 1:

```
flint-realtime-fabric/
├── Cargo.toml        # single-crate binary, edition 2024 — REPLACE with [workspace]
├── src/main.rs       # hello-world — REMOVE; the root is a workspace, not a bin
├── .gitignore        # /target
└── docs/
    ├── IMPLEMENTATION-PLAN.md                          # RFC-FRF-002 (authoritative)
    └── flint-realtime-fabric-implementation-plan.html # same content, rich format
```

**First action (Phase 0, step 1):** convert the root `Cargo.toml` to a virtual
workspace manifest and delete `src/main.rs`. The root crate is a workspace, not a
binary. Keep `edition = "2024"` across member crates.

---

## Architecture in one paragraph

Feature-based hexagonal. `frf-domain` holds pure types (serde only, no I/O).
`frf-ports` defines trait seams (`LogBroker`, `AuthzProvider`, `IdentityVerifier`,
`CrdtStore`, `MediaSignaler`, `FederationBridge`). `frf-app` orchestrates
use-cases against ports only. Every `frf-*` adapter crate implements exactly one
port and is selected by a Cargo feature, so a deployment compiles only the planes
it runs. `frf-gateway` (Axum 0.8.8) is the deployable: WebSocket mux + tonic gRPC
+ Connect. The dependency rule is absolute and points inward — domain ← app ←
infrastructure/interface. Nothing in domain or app may import an adapter crate.

See `docs/IMPLEMENTATION-PLAN.md` §02 for the full crate graph and §01 for the
target repository tree.

---

## Non-negotiable rules for code generation

These are hard quality gates. A violation is a failure, not a style nit.

1. **Contract first.** Author `proto/flint/v1/*.proto` and freeze it (`git tag
   proto-v1`) before generating or hand-writing any SDK. Generated SDKs built
   before the freeze churn endlessly. Breaking changes are a new proto version,
   never an edit to v1.
2. **Dependency rule.** `frf-domain` imports nothing but serde. `frf-app` imports
   only `frf-domain` + `frf-ports`. Adapters depend inward. The compiler should
   make a violation impossible — keep adapter crates out of app/domain `[dependencies]`.
3. **No `unwrap()` / `expect()` in library crates.** `thiserror` for library
   errors; `anyhow` only at binary edges (`frf-gateway`, `frf-cli`). Clippy-deny it.
4. **Lint gate.** `clippy::pedantic` + `deny(warnings)` must pass. Pin MSRV.
5. **Public API hygiene.** `#[non_exhaustive]` on public enums; newtype IDs with
   `#[repr(transparent)]` over bare `String`; semver discipline on `frf-domain`
   and every SDK.
6. **Observability.** `tracing` spans across every port boundary call.
7. **One port per adapter.** Do not let an adapter crate implement two ports or
   reach across to another adapter. Composition happens in `frf-gateway`.
8. **Naming.** kebab-case crate dirs and file names; Rust module files are
   snake_case (language constraint). TSX over JSX in any generated TS UI.

---

## Phase 0 — exact build order for Claude Code

Do these in sequence. Stop after Phase 0 for approval before Phase 1.

1. **Workspace.** Replace root `Cargo.toml` with a `[workspace]` manifest
   (resolver = "2"), a `[workspace.package]` block (edition 2024, shared
   version/license/repo), and `[workspace.dependencies]` pinning the shared stack:
   `tokio`, `axum = "0.8.8"`, `tonic`, `prost`, `ractor`, `serde`, `serde_json`,
   `bytes`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `uuid`,
   `chrono`, `dashmap`. Add the Iggy fork:
   `iggy = { git = "https://github.com/GQAdonis/iggy", branch = "master" }`.
   Delete `src/main.rs`.
2. **Domain.** Create `crates/frf-domain` with the types from §02: `EventEnvelope`,
   `Channel`, `Offset`, `Cursor`, `EntityChange`, `AgentEvent`, `SyncOp`,
   `Presence`, `SignalEnvelope`, newtype IDs. Serde derives only. Unit tests for
   (de)serialization round-trips.
3. **Ports.** Create `crates/frf-ports` with the six traits from §02 as
   `async_trait` (or native async-in-trait if MSRV allows). No implementations.
4. **Contract.** Author `proto/flint/v1/{envelope,entity,agent,signal,sync,authz}.proto`
   per §03. Create `crates/frf-proto` with a `build.rs` running `tonic-build`.
   Confirm it compiles. Tag `proto-v1`.
5. **Gateway stub.** Create `crates/frf-gateway` (Axum 0.8.8) that boots, exposes
   `/healthz`, and mounts an empty tonic service + a WS upgrade handler that
   echoes. No business logic yet.
6. **CI.** Create `dagger/` pipelines: `fmt --check`, `clippy --all-targets
   -- -D warnings -W clippy::pedantic`, `test`, MSRV check. Wire it to run on push.

**Phase 0 exit criteria:** workspace compiles; `proto-v1` tagged; CI green;
`frf-gateway` serves `/healthz`. Then halt for approval.

---

## Stack reference (confirm currency at kickoff)

| Concern | Choice |
|---|---|
| Web / gateway | Axum 0.8.8 |
| gRPC | tonic + prost |
| Actors (BossFang) | ractor |
| Event spine | Apache Iggy (GQAdonis fork) behind `LogBroker` |
| Identity | Ory Kratos / Oathkeeper (JWT) |
| AuthZ (ReBAC / RLS) | Ory Keto (Zanzibar) |
| Action policy | Cedar (existing PAUX-1) |
| CRDT engine | Loro **or** automerge-rs — **OPEN, decide before Phase 3** |
| On-device store | redb |
| Server store | SurrealDB 3.x |
| Postgres CDC | logical replication slot |
| Media SFU | str0m (sovereign) / LiveKit (hosted) |
| Federation | Tuwunel (Matrix) · Tranquil (ATProto) |
| FFI bindings | UniFFI (Swift, Kotlin) · flutter_rust_bridge (Dart) |
| Browser transport | Connect-ES + WS mux |
| CI | Dagger |

> Versions for UniFFI, flutter_rust_bridge, Connect, and tonic shift. Confirm
> current releases and language coverage before committing the FFI/codegen
> approach (Risks table, `docs/IMPLEMENTATION-PLAN.md` §09).

---

## SDK strategy (why nine languages is tractable)

Only **Rust** is hand-written. **Go / C# / browser-TS** are generated from the
frozen proto. **Swift / Kotlin / Dart** bind to the same Rust core over FFI, which
also gives every device identical offline CRDT merge. `prometheus-entity-management`
is a thin `RealtimeAdapter` on the TS SDK, not a separate SDK. Java-for-Android
consumes the UniFFI Kotlin binding — do not hand-write a separate Java SDK. Full
table in §04.

---

## Documents

| File | What |
|---|---|
| `docs/IMPLEMENTATION-PLAN.md` | RFC-FRF-002 — authoritative build plan |
| `docs/flint-realtime-fabric-implementation-plan.html` | Same, rich format |
| `docs/flint-realtime-fabric-architecture.*` | RFC-FRF-001 — plane-separation design (add if not present) |

---

## License

MIT (workspace default). Confirm before first publish.
