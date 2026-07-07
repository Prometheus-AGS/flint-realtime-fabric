# Plan — phase-16-production-hardening

> Backend: openspec
> Authored: 2026-07-06
> Source: `assessment.md` (baseline) + phase-15 production-readiness audit
> Scope decisions (operator, 2026-07-06):
> - **str0m SFU: DEFER** — gate `SFU_MODE=sovereign` off, LiveKit-hosted is the only v1 media path.
> - **Federation (Matrix/ATProto): DEFER** — mark unsupported for v1; fix tenant-ID wiring only.
> - **`unwrap_used`: workspace-wide** — enable, fix crdt/redb, `#[allow]` + justify the rest.

## Ordering rationale

Changes are ordered by dependency and severity. **Track G1 (security) lands first and
in full — no G2+ change starts while a G1 CRITICAL is open.** Within a track, changes are
independent unless a `depends-on` is noted. Each change is one coherent, verifiable unit.

Total: **26 changes** across 5 tracks. First to apply: **p16-c001** (compose auth bypass).

---

## Track G1 — Security boundary (7 changes · do first)

| ID | Change | Goal | Files | Agent | Risk |
|----|--------|------|-------|-------|------|
| **p16-c001** | Remove auth bypass from production-shaped `compose.yml`; keep it in `compose.ci.yml` + `compose.override.yml` (dev/CI only); make `dev_no_auth()` compile-gated so it cannot exist in a default release build | G1.1 (C1) | `compose.yml`, `crates/frf-gateway/src/config.rs`, `routes/{publish,subscribe}.rs` | rust-reviewer | LOW |
| **p16-c002** | App-layer tenant-equality assertion: reject when JWT `tenant_id` ≠ target channel/envelope `tenant_id` on publish + subscribe | G1.2 (H1) | `crates/frf-app/src/{publish,subscribe}.rs`, `frf-app/src/error.rs` | tdd-guide | LOW |
| **p16-c003** | Enable `clippy::unwrap_used` workspace-wide; fix ~50 library sites (frf-crdt, frf-store-redb); `#[allow]` + justify truly-unreachable cases; wire the lint into CI (`ci.yml` + `dagger`) | G1.3 (H12) | workspace `Cargo.toml`, `crates/frf-crdt/*`, `crates/frf-store-redb/*`, `.github/workflows/ci.yml`, `dagger/codegen.ts` | rust-reviewer | MEDIUM |
| **p16-c004** | Verify JWT issuer (`iss`) in identity verifier against configured expected issuer | G1.4 (#25) | `crates/frf-identity-ory/src/verifier.rs`, `config` | security-reviewer | LOW |
| **p16-c005** | Wire `tower-http`: rate limiting, request body-size limit, explicit CORS policy on the gateway router | G1.5 (H2) | `crates/frf-gateway/src/lib.rs`, `Cargo.toml` | rust-reviewer | LOW |
| **p16-c006** | Externalize the flint-gate signing secret out of compose into env/secret-manager; document rotation | G1.6 (#48) | `compose*.yml`, docs | security-reviewer | LOW |
| **p16-c007** | Make Cedar functional or explicit no-op: supply real `Entities` + a matchable policy set, or gate Cedar off with a clear "disabled" log — no silent allow-all/deny-all | G1.7 (H3) | `crates/frf-policy-cedar/src/lib.rs`, `policy.cedar` | rust-reviewer | MEDIUM |

**G1 exit:** No release artifact bypasses auth · tenant-equality enforced in code ·
`unwrap_used` green in CI with zero un-justified library unwraps · `iss` verified ·
rate-limit/CORS live · secret externalized · Cedar honest.

---

## Track G2 — Make planes real (3 changes · reduced by scope decisions)

| ID | Change | Goal | Files | Agent | Risk |
|----|--------|------|-------|-------|------|
| **p16-c008** | Wire admin-UI ↔ gateway transport end to end: serve Connect/gRPC-web for SpineService (`tonic_web` + `accept_http1`), add real `/ws/v1/signal` route, embed admin-ui via `rust-embed`/`ServeDir`, add login/auth flow so calls carry a real JWT, reconcile default gateway URL/port across UI + gateway | G2.2 (C2/C3/H9/H10/H11) | `crates/frf-gateway/src/{lib,main}.rs`, `routes/`, `admin-ui/src/{infrastructure,features/auth}` | code-architect | MEDIUM |
| **p16-c009** | Federation honesty (DEFER protocol work): mark Matrix inbound + ATProto outbound unsupported for v1 — stop wiring broken directions as bidirectional; return a clear "unsupported" rather than silent empty/`Err`; document deferral | G2.5 (#10/#27) | `crates/frf-gateway/src/main.rs`, bridge crates, docs | rust-reviewer | LOW |
| **p16-c010** | str0m defer + LiveKit correctness: gate `SFU_MODE=sovereign` off by default (LiveKit-hosted only for v1) and fix LiveKit cross-node listening (H4); document str0m as future work | G2.1/G2.3 (C4/H4) | `crates/frf-gateway/src/main.rs`, `crates/frf-media-livekit/src/adapter.rs`, config, docs | rust-reviewer | MEDIUM |

> **Deferred to future phase (recorded, not lost):** real str0m WebRTC SFU; Matrix
> inbound sync + ATProto outbound send protocol impls. G2.4 (federation tenant IDs)
> folds into c009 where cheap.

**G2 exit:** every plane the gateway boots either functions end to end or is honestly
labeled unsupported/disabled. Admin-UI connects to a real gateway.

---

## Track G3 — Missing deliverables (6 changes)

| ID | Change | Goal | Files | Agent | Risk |
|----|--------|------|-------|-------|------|
| **p16-c011** | Create `frf-sdk-rust` — hand-written Rust client (connect/publish/subscribe), the single home for connection lifecycle | G3.1 (C5) | `crates/frf-sdk-rust/*`, workspace `Cargo.toml` | code-architect | MEDIUM |
| **p16-c012** | Reconnection/backoff/replay-from-offset in `frf-sdk-rust` using resumable-offset proto primitives; surface through TS/Go/C# | G3.3 (H6) | `crates/frf-sdk-rust/*`, `sdks/{ts,go,csharp}` | rust-reviewer | MEDIUM · depends-on c011 |
| **p16-c013** | Bind remaining 5 proto services (sync, agent, signal, entity, authz) in real SDKs; register missing servers in gateway | G3.4 (H7) | `sdks/{ts,go,csharp}`, `crates/frf-gateway/src/*` | rust-reviewer | MEDIUM |
| **p16-c014** | Extend `frf-ffi` beyond 3 CRDT fns to real transport (connect/auth/subscribe/publish) for Swift/Kotlin/Dart; fix empty Dart bridge dir so package compiles | G3.2 (C6) | `crates/frf-ffi/src/lib.rs`, `sdks/{swift,kotlin,dart}` | code-architect | HIGH · depends-on c011 |
| **p16-c015** | Create `frf-cli` — seed Keto tuples, manage CDC slots, inspect broker offsets, force checkpoints; surface `AuthzProvider::write()` | G3.5 (H13) | `crates/frf-cli/*`, workspace `Cargo.toml` | code-architect | MEDIUM |
| **p16-c016** | Stop C# SDK forking its own proto copy; generate from frozen `proto-v1` | G3.6 (#32) | `sdks/csharp/*` | csharp-reviewer | LOW |

**G3 exit:** `frf-sdk-rust` + `frf-cli` exist and are usable · all 6 proto services
reachable through Rust + TS SDKs · reconnection implemented once.

---

## Track G4 — Operability (5 changes)

| ID | Change | Goal | Files | Agent | Risk |
|----|--------|------|-------|-------|------|
| **p16-c017** | Real `/readyz` probing Iggy/Keto/JWKS liveness; keep `/healthz` as liveness; update compose/K8s `depends_on` to gate on readiness | G4.1 (H8) | `crates/frf-gateway/src/routes/health.rs`, `compose*.yml` | rust-reviewer | LOW |
| **p16-c018** | `/metrics` Prometheus endpoint alongside tracing spans | G4.2 (#34) | `crates/frf-gateway/src/{lib,routes}.rs`, `Cargo.toml` | rust-reviewer | LOW |
| **p16-c019** | Graceful shutdown on SIGTERM — drain in-flight requests + WS streams | G4.3 (#35) | `crates/frf-gateway/src/main.rs` | rust-reviewer | LOW |
| **p16-c020** | Semantic config validation at boot (e.g. hosted SFU must not start with empty LiveKit creds) | G4.4 (#36) | `crates/frf-gateway/src/config.rs`, `main.rs` | rust-reviewer | LOW |
| **p16-c021** | Keto schema-migration step so persistent-DSN deployments start | G4.5 (#37) | `compose*.yml`, ops docs | devops-engineer | LOW |

**G4 exit:** dead dependency → pod out of rotation · SIGTERM drains cleanly ·
misconfig fails fast · gateway observable via `/metrics`.

---

## Track G5 — Documentation & release artifacts (5 changes)

| ID | Change | Goal | Files | Agent | Risk |
|----|--------|------|-------|-------|------|
| **p16-c022** | Env-var reference + `.env.example` — all ~30 gateway vars, which are secrets, which are dev-only bypasses | G5.1 (H14) | `docs/`, `.env.example` | doc-updater | LOW |
| **p16-c023** | Deployment/operations runbook — prod topology, secret provisioning, CDC-slot lifecycle/recovery, scaling, upgrade/rollback | G5.2 (H15) | `docs/` | devops-engineer | LOW |
| **p16-c024** | Security-model document — auth boundary, tenant isolation, Keto/Cedar roles | G5.3 (#41) | `docs/` | security-reviewer | LOW |
| **p16-c025** | Fix stale README — remove references to now-existing crates, update phase status | G5.4 (#42/#43) | `README.md` | doc-updater | LOW · depends-on c011,c015 |
| **p16-c026** | Add LICENSE file (MIT), CONTRIBUTING, CHANGELOG, SECURITY.md, proto-derived API reference, ADR for UniFFI/frb/Connect/tonic versions | G5.5 (#54/#57/#58/#59) | repo root, `docs/decisions/` | doc-updater | LOW |

**G5 exit:** an SRE and an external contributor can each stand up and operate the system
from docs alone.

---

## Phase completion criteria

- All CRITICAL (7) + HIGH (15) phase-15 findings closed or honestly deferred (str0m,
  federation protocol work) with rationale.
- Re-run of the production-readiness audit returns **zero CRITICAL** and materially
  fewer HIGH.
- `cargo check --workspace`, `cargo clippy -- -D warnings -W clippy::pedantic
  -W clippy::unwrap_used`, `cargo fmt --check` all green in CI.

## Recommended apply order

p16-c001 → c002 → c003 → c004 → c005 → c006 → c007  (G1 complete — gate)
→ c008 → c009 → c010  (G2)
→ c011 → c012 → c013 → c014 → c015 → c016  (G3)
→ c017 → c018 → c019 → c020 → c021  (G4)
→ c022 → c023 → c024 → c025 → c026  (G5)

Apply with `/kbd-apply` (KBD-owned, one change at a time) or `/opsx:apply <change-id>`.
