# Plan: Phase 1 — Core Infrastructure Adapters

> **RFC-FRF-002 · Prometheus AGS**
> Status: plan complete · 2026-06-17
> Prepared by: kbd-plan workflow

---

## Objective

Wire the first real infrastructure behind the Phase 0 port seams, culminating in a demonstrable end-to-end flow:

```
Postgres WAL → CDC capture → Iggy spine → Keto RLS check → WebSocket fan-out to verified subscriber
```

---

## Ordered Changes

### p1-c001 — Workspace Expansion
**Agent:** rust-build-resolver  
**Rationale:** All subsequent changes need the 5 new workspace members and shared dependency pins in place before they can compile. This is the mandatory prerequisite for every other change in this phase.  
**Dependencies:** none  
**Risk:** Low — purely additive Cargo.toml edits and stub crate scaffolding.

### p1-c002 — frf-app (Use-Cases)
**Agent:** rust-reviewer  
**Rationale:** `SubscribePipeline` and `PublishUseCase` must exist as the coordination layer before the gateway can wire adapters through them. Implementing this generic layer forces the port seams to be correct before adapters are written.  
**Dependencies:** p1-c001  
**Risk:** Medium — generic bounds over `<L: LogBroker, A: AuthzProvider, I: IdentityVerifier>` must be correct; borrow checker will catch violations.

### p1-c003 — frf-broker-iggy (LogBroker → Apache Iggy)
**Agent:** rust-reviewer  
**Rationale:** The most complex adapter — Iggy's poll-based consumer must be wrapped in a bounded mpsc channel to produce the `EventStream` type. Must be done before the gateway can demonstrate real event delivery.  
**Dependencies:** p1-c001  
**Risk:** High — Iggy API surface uses the GQAdonis fork; the `IggyClient::from_connection_string` async API and `producer/consumer` pattern must be validated against the actual fork. The T7 integration test (marked `#[ignore]`) gates this.  
**Note:** Run `T1` first — audit the fork's public API before writing any adapter code.

### p1-c004 — frf-authz-keto (AuthzProvider → Ory Keto)
**Agent:** rust-reviewer  
**Rationale:** Per-event RLS enforcement is a hard architectural requirement. The DashMap cache with 60s TTL is the critical scaling mitigation for O(subscribers × events) Keto call volume.  
**Dependencies:** p1-c001  
**Risk:** Medium — Keto REST API is well-documented; the cache invalidation logic (`invalidate_object` on delete) is the subtle part and requires careful unit testing.

### p1-c005 — frf-identity-ory (IdentityVerifier → Oathkeeper)
**Agent:** rust-reviewer  
**Rationale:** JWT verification is the gateway boundary. The JWKS cache with key-rotation retry (refresh on unknown `kid`, retry once) must be correct before any authenticated endpoint can function.  
**Dependencies:** p1-c001  
**Risk:** Medium — `jsonwebtoken` RS256 verification with JWKS is well-trodden; the httpmock test with a synthetic RSA keypair is the validation gate.

### p1-c006 — frf-postgres-cdc (WAL Logical Replication → Spine)
**Agent:** rust-reviewer  
**Rationale:** The CDC-to-spine path completes the write side of the demo. `PostgresCdcConsumer` receives `Box<dyn LogBroker>` via DI (never imports `frf-broker-iggy` directly) — this enforces the one-adapter-per-port rule at the boundary.  
**Dependencies:** p1-c001, p1-c003 (LogBroker trait — already in frf-ports from Phase 0)  
**Risk:** Medium-high — `tokio-postgres` logical replication with `pgoutput` is niche; the `decode.rs` unit tests with synthetic WAL structures are the critical verification path.

### p1-c007 — frf-gateway: Subscription Mux + Publish Endpoint
**Agent:** rust-reviewer  
**Rationale:** This is the composition root — the only crate allowed to import all concrete adapters. Wires `AppState`, the WebSocket subscribe handler, and the `POST /v1/publish` endpoint. When this change passes its integration test (marked `#[ignore]`), Phase 1 exit criterion is met.  
**Dependencies:** p1-c001, p1-c002, p1-c003, p1-c004, p1-c005  
**Risk:** Medium — Axum 0.8.8 WebSocket + `axum::extract::State` wiring is well-documented; the subtle parts are correct error-to-HTTP-status mapping before the WS upgrade and clean stream teardown on disconnect.

---

## Parallelization Opportunity

Changes p1-c003, p1-c004, and p1-c005 have no dependency on each other — only on p1-c001. They can be implemented in parallel once the workspace expansion lands:

```
p1-c001 → p1-c002 ─────────────────────────────────┐
p1-c001 → p1-c003 (Iggy broker) ───────────────────┤
p1-c001 → p1-c004 (Keto authz) ────────────────────┤→ p1-c007 (gateway)
p1-c001 → p1-c005 (Ory identity) ──────────────────┤
p1-c001 → p1-c006 (Postgres CDC) ──────────────────┘
```

---

## Open Decisions (Carryover from Assessment)

| Decision | Required Before | Status |
|---|---|---|
| CRDT engine: Loro vs automerge-rs | Phase 3 | Open |
| UniFFI / flutter_rust_bridge version | Phase 3 kickoff | Open |
| Connect-ES version | Phase 2 kickoff | Open — v2.1.2 confirmed for TS |
| Tonic version | Phase 0 kickoff | Resolved: 0.14.5 (current) |

---

## Phase 1 Exit Criteria

1. `cargo check --workspace` exits 0
2. `cargo test --workspace` (non-ignored) passes — including frf-app, frf-authz-keto, frf-identity-ory unit tests
3. `cargo clippy --workspace -- -D warnings -W clippy::pedantic` exits 0
4. `cargo fmt --check --all` exits 0
5. `frf-broker-iggy` compiles with `[features] iggy-test-doubles` enabled (no live Iggy required)
6. `frf-authz-keto` httpmock tests pass (5 tests, all green)
7. `frf-identity-ory` httpmock tests pass (4 tests, all green)
8. `frf-postgres-cdc` WAL decode unit tests pass
9. `frf-gateway` serves `/healthz`, `/ws/v1/subscribe`, and `/v1/publish` routes (compile-verified; integration test marked `#[ignore]`)

---

## Recommended First Command

```
/kbd-execute phase-1-core-infrastructure-adapters p1-c001-workspace-expansion
```
