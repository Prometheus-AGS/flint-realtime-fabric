# Reflection: Phase 0 — Flint Realtime Fabric Foundations

> RFC-FRF-002 · Prometheus AGS
> Reflected: 2026-06-17
> Status: **COMPLETE** — all exit criteria met

---

## Goal Achievement

| Goal | Status | Notes |
|---|---|---|
| Implement the ultimate realtime communications, pub/sub, and agentic application support framework | **PARTIAL** | Phase 0 lays the foundational scaffold. The framework itself is built across Phases 1–7. This goal is a multi-phase target, not a Phase 0 deliverable. |
| Research and compete with Supabase Realtime, Convex, Elixir Phoenix, LiveKit, Matrix, and other frameworks | **MET** | Full competitive analysis matrix produced in `assessment.md` covering 9 competitors across 8 capability dimensions (~120 feature cells). Seven unique FRF differentiators documented. |
| Support WebRTC, messaging, agent2agent, AG-UI, AT Protocol, enterprise pub/sub at scale | **PARTIAL** | Port trait seams (LogBroker, AuthzProvider, IdentityVerifier, CrdtStore, MediaSignaler, FederationBridge) define the capability contracts. Implementations begin Phase 1. |
| Produce competitive analysis matrix as part of the assessment phase | **MET** | `assessment.md` §3a–3h contains the full matrix. |

**Overall achievement: 2 of 4 goals fully met; 2 are multi-phase goals where Phase 0 delivered the foundation artifacts (MET for Phase 0 scope).**

---

## Phase 0 Exit Criteria

| Criterion | Status |
|---|---|
| Workspace compiles (`cargo check --workspace`) | ✅ PASS |
| `proto-v1` git tag exists and contract is frozen | ✅ PASS |
| `frf-gateway` serves `GET /healthz → 200` (verified by `axum-test`) | ✅ PASS |
| CI gates green (fmt, clippy::pedantic, test, msrv) | ✅ PASS — all 4 gates green locally; `.github/workflows/ci.yml` authored |

---

## Delivered Changes

| Change | Summary | Files |
|---|---|---|
| `p0-c001-workspace-restructure` | Virtual Cargo workspace (resolver=2, edition 2024, MSRV 1.85); `[workspace.dependencies]` pins full stack | `Cargo.toml`, `rust-toolchain.toml` |
| `p0-c002-frf-domain` | Pure domain types: 7 newtype IDs, Channel/Offset/Cursor/EventEnvelope, EventKind, EntityChange, AgentEvent, SyncOp, Presence, SignalEnvelope; all enums `#[non_exhaustive]`; 10 serde roundtrip tests | `crates/frf-domain/src/*`, `crates/frf-domain/tests/serde_roundtrip.rs` |
| `p0-c003-frf-ports` | Six async port trait seams (LogBroker, AuthzProvider, IdentityVerifier, CrdtStore, MediaSignaler, FederationBridge); typed PortError; no implementations | `crates/frf-ports/src/*` |
| `p0-c004-frf-proto` | Six proto files (`envelope`, `entity`, `agent`, `signal`, `sync`, `authz`); `prost-build` codegen in `frf-proto/build.rs`; contract frozen at `proto-v1` git tag | `proto/flint/v1/*.proto`, `crates/frf-proto/*` |
| `p0-c005-frf-gateway-stub` | Axum 0.8.8 gateway binary + library split; `/healthz` → `{"status":"ok"}`; WS echo at `/ws`; `axum-test` integration test passes | `crates/frf-gateway/src/*`, `crates/frf-gateway/tests/health.rs` |
| `p0-c006-dagger-ci` | `.github/workflows/ci.yml` with 4 jobs (fmt, clippy-pedantic, test, msrv); `rust-toolchain.toml` pins stable; `dagger/` placeholder for Phase 1 SDK migration | `.github/workflows/ci.yml`, `dagger/README.md` |

---

## Artifact Quality Summary

No artifact-refiner QA gate was run (`.refiner/artifacts/` directory does not exist). The quality bar was enforced directly through the four CI gates:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | ✅ PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` | ✅ 0 warnings, 0 errors |
| `cargo test --workspace` | ✅ 11 tests pass (10 serde roundtrips + 1 healthz) |
| MSRV (rust 1.85 — edition 2024 minimum) | ✅ PASS |

**Note:** Two formatting corrections were applied during QA (long lines in `ids.rs` and `serde_roundtrip.rs` wrapping to satisfy `rustfmt`). Both are cosmetic; no logic changed.

---

## Technical Debt Introduced

| Item | Severity | Phase to Resolve |
|---|---|---|
| Dagger SDK pipeline deferred | LOW | Phase 1 — `dagger/` directory scaffolded; Cargo.toml and `main.rs` not yet written because `dagger-sdk` crate flux during Phase 0. Current GitHub Actions CI is functionally equivalent. |
| `tower-http` added to workspace but not used in Phase 0 | LOW | Will be consumed by Phase 1 middleware (auth, tracing, rate-limiting). No action needed. |
| `iggy` Git dependency (GQAdonis fork) not yet used | LOW | Phase 1 implements `frf-broker-iggy`. `LogBroker` port is ready. |
| No `#[tracing::instrument]` on port trait declarations | NOTE | By design: `#[tracing::instrument]` cannot be applied to trait declarations with default-less methods. Adapter implementations must instrument their impls. Documented in `frf-ports` doc comments. |
| `frf-gateway` has no gRPC service registered | LOW | Phase 1 — tonic `Server::builder()` is present via dependency. Service registration happens when the first port adapter is wired. |

---

## Lessons Captured

1. **prost-build vs tonic-build at tonic 0.14:** `tonic-build 0.14` dropped `compile_protos()` — it is now pure service codegen. Multi-file proto compilation requires `prost-build::compile_protos(&[...], &[include_dir])` directly. Future phases should use `prost-build` for message types and invoke `tonic-build::configure()` only for service stubs when needed.

2. **Absolute path resolution in `build.rs`:** `env!("CARGO_MANIFEST_DIR")` is the only reliable way to locate workspace-root proto files from a crate's `build.rs`. The build script's working directory is not guaranteed to be the workspace root.

3. **Edition 2024 requires MSRV 1.85:** Attempting MSRV 1.82 with `edition = "2024"` fails at compile time. The minimum for edition 2024 is Rust 1.85.

4. **tonic 0.14 TLS feature is `tls-ring`, not `tls`:** The feature rename is a breaking change from prior versions. `features = ["transport", "tls"]` silently adds nothing; `features = ["transport", "tls-ring"]` is the correct form.

5. **`#[tracing::instrument]` on trait declarations:** Attribute macros that reference `self` cannot appear on trait methods without default bodies. The correct pattern is to document the instrumentation contract and enforce it in adapter code reviews.

6. **Binary-crate integration tests need a `[lib]` section:** Rust integration tests (in `tests/`) can only import from library crates. A binary-only `frf-gateway` cannot be tested with `axum-test`. Splitting into `[lib]` (exports `build_router()`) + `[[bin]]` (calls the lib) is the canonical pattern.

7. **`#[non_exhaustive]` on all public enums:** Applying this discipline from day one prevents ecosystem lock-in. All `EventKind`, `AgentEventKind`, `SyncOpKind`, `PresenceStatus`, `SignalKind`, `SfuMode`, `ChangeOp` enums use `#[non_exhaustive]`, matching the CLAUDE.md requirement.

---

## Open Decisions (Unresolved — Must Be Addressed Before Stated Phase)

| Decision | Must Be Made Before | Current State |
|---|---|---|
| CRDT engine: Loro vs automerge-rs | Phase 3 kickoff | Open. Both are mature Rust crates. `SyncOp::payload: Vec<u8>` is engine-agnostic; decision affects `frf-crdt` adapter only. |
| UniFFI version + language coverage | Phase 3 kickoff | Not yet verified. Confirm before `frf-ffi` planning. |
| flutter_rust_bridge version | Phase 3 kickoff | Not yet verified. Confirm before `frf-ffi` planning. |
| Connect-ES version for browser transport | Phase 2 kickoff | Not yet verified. Affects `frf-wasm` and `admin-ui` SDK config. |
| Tonic version pin (currently 0.14) | Phase 1 kickoff | Verify 0.14 is still current stable; check for 0.15 breaking changes if released. |

---

## Risk Register — Phase 0 Delta

| Risk | Status | Notes |
|---|---|---|
| Iggy clustering not yet available | UNCHANGED | `LogBroker` port provides safe migration path. Accepted for Phase 0–2. |
| CRDT engine decision | UNCHANGED | Still open. Documented above. |
| Per-event Keto check latency | UNCHANGED | Design begins Phase 1. Subscribe-time scoping planned. |
| Proto stability after freeze | MITIGATED | `proto-v1` tagged. Breaking changes are a new version. |
| str0m maturity | UNCHANGED | LiveKit adapter as fallback. Addressed in Phase 4. |

---

## Recommended Next Phase

**Phase 1: Core Infrastructure Adapters**

Priority order based on dependency graph and Phase 0 learnings:

1. **`frf-broker-iggy`** — Implement `LogBroker` against the GQAdonis Iggy fork. This unlocks the durable spine that all other adapters and the gateway depend on. Without it, no fan-out, no CDC, no presence, no agent events.

2. **`frf-authz-keto` + `frf-identity-ory`** — Implement `AuthzProvider` and `IdentityVerifier`. The security foundation must be in place before any user-facing subscription logic is exposed.

3. **`frf-gateway` subscription mux** — Wire the first real subscription path: `LogBroker.subscribe()` → WS fan-out → Keto RLS check per event. This is the core value proposition and the first integration test of the hexagonal architecture.

4. **Postgres CDC adapter stub** — `frf-postgres-cdc` can begin without the Iggy adapter because it produces `EventEnvelope` values; only the publish path needs `LogBroker` to be live.

**Phase 1 has no open decisions blocking it.** The single version to confirm at kickoff is tonic 0.14 currency.

---

## Summary

Phase 0 is complete. The hexagonal architecture scaffold is in place, the protobuf contract is frozen and tagged, the domain model compiles and roundtrips cleanly, and all four CI quality gates pass. The competitive analysis confirms the FRF design is the only sovereign open-source substrate that covers the full target capability matrix.

The six changes introduced no architectural violations. The dependency rule (domain ← ports ← adapters ← gateway) is enforced structurally: no adapter crates exist yet, and `frf-domain`/`frf-ports`/`frf-gateway` `[dependencies]` contain no cross-layer adapter imports.

Phase 1 is unblocked.
