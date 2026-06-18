# Reflection: Phase 1 — Core Infrastructure Adapters

> **RFC-FRF-002 · Prometheus AGS**
> Status: reflect complete · 2026-06-18
> Phase: phase-1-core-infrastructure-adapters

---

## 1. Goal Achievement

| Goal | Status | Notes |
|---|---|---|
| Implement `frf-broker-iggy`: LogBroker port backed by Apache Iggy (GQAdonis fork) | ✅ MET | Full `publish`, `subscribe`, `seek`, `ack`, `ensure_channel` impl; `ReceiverStream` wraps Iggy consumer poll loop; 4 unit tests + 1 `#[ignore]` integration test |
| Implement `frf-authz-keto`: AuthzProvider port backed by Ory Keto | ✅ MET | `check` (cache-aside DashMap, 60s TTL), `write` (PUT), `delete` (DELETE + cache invalidate); 5 httpmock integration tests pass |
| Implement `frf-identity-ory`: IdentityVerifier port backed by Ory Oathkeeper JWT | ✅ MET | JWKS cache (`Arc<RwLock<Option<JwkSet>>>`), auto-refresh on unknown kid or signature error, claims mapping; 4 tests including key-rotation retry path |
| Wire first live subscription path in `frf-gateway` | ✅ MET | `/ws/v1/subscribe` — JWT auth + Keto check before WS upgrade; `EventStream` forwarded as JSON text frames; `/v1/publish` wired; `AppState<L,A,I>` fully composed |
| **Begin** `frf-postgres-cdc`: WAL logical replication → EventEnvelope facts | ⚠️ PARTIAL | Scaffolded but not functional. WAL decode unit tests pass (6 tests on `Relation`/`Column` types). The consumer loop body is `#[allow(dead_code)]` — `tokio-postgres 0.7` has no logical replication API. The goal was "Begin" (scaffolded, decode path proven), which is technically met; however the crate does not yet produce `EventEnvelope` facts onto the spine. See Gate Question in §7. |
| Confirm version currency: tonic 0.14, Connect-ES, Iggy fork | ✅ MET | tonic 0.14.6 confirmed; Iggy GQAdonis fork API audited in p1-c003; Connect-ES confirmed Phase 2 dependency; Keto REST API confirmed no Rust crate → reqwest adapter |

**Phase goal achievement: 5.5/6 (~92%)**

- 5 goals fully met
- 1 goal partially met (`frf-postgres-cdc` scaffolded; functional loop not yet wired)
- 0 goals not met

The IMPLEMENTATION-PLAN §08 Phase 1 exit criterion is: *"Supabase-like entity sync end-to-end with RLS; Rust client; cache invalidation working."* CDC is a prerequisite for the full end-to-end path. The subscribe → RLS → fan-out path is working; the Postgres → spine ingestion path is scaffolded but not functional. **This is a partial exit. See §7 gate question.**

---

## 2. Delivered Changes

| Change | Commit | Artifacts |
|---|---|---|
| p1-c001-workspace-expansion | `fd12a47` | 5 new adapter crate stubs; workspace deps expanded; `cargo check --workspace` green |
| p1-c002-frf-app | `9ee6547` | `SubscribePipeline<L,A,I>`, `PublishUseCase<L,I>`, `AppError`; 7 tests (verify, authz, broker, RLS filter) |
| p1-c003-frf-broker-iggy | `1a4f6b6` | `IggyBroker` implementing `LogBroker`; channel mapping (`tenant_id` → stream, `channel.path` → topic); 8 tests |
| p1-c004-frf-authz-keto | `917af7c` | `KetoAuthzProvider` with DashMap cache; `CacheEntry` with TTL; 8 tests |
| p1-c005-frf-identity-ory | `4fc4a57` | `OryIdentityVerifier`; JWKS fetch/cache/refresh; `FrfClaims` → `VerifiedClaims` mapping; 4 tests |
| p1-c006-frf-postgres-cdc | `abc6587` | `CdcConfig`, `Relation`/`Column` types, `decode_insert/update/delete`; `PostgresCdcConsumer` skeleton; 6 unit decode tests + 1 ignored integration test |
| p1-c007-gateway-subscription-mux | `bfa764b` | `GatewayConfig::from_env()`, `GatewayError` with `IntoResponse`, `ws_subscribe`, `publish_event`, `AppState<L,A,I>`, `build_router()`; updated health test; smoke test |

**Total: 7/7 changes delivered (1 delivering a skeleton, not a working implementation).**

---

## 3. Artifact Quality Summary

No artifact-refiner logs were generated for this phase (`.refiner/artifacts/` is absent — refiner QA gate was not wired into the kbd-execute flow for Phase 1).

All changes were verified manually against the CI gate:

| Gate | Result |
|---|---|
| `cargo fmt --check --all` | ✅ PASS |
| `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` | ✅ PASS (0 warnings) |
| `cargo test --workspace` | ✅ PASS (0 failures, 3 ignored integration tests requiring live infra) |
| `cargo check --workspace` | ✅ PASS |

**Test counts (non-ignored, by crate):**

| Crate | Tests |
|---|---|
| frf-domain | 10 (serde roundtrip) |
| frf-app | 7 (pipeline unit tests) |
| frf-broker-iggy | 8 (channel mapping unit + integration) |
| frf-authz-keto | 8 (httpmock integration) |
| frf-identity-ory | 4 (JWKS mock server) |
| frf-postgres-cdc | 6 (WAL decode unit — does **not** test live replication) |
| frf-gateway | 1 (healthz) |
| **Total** | **44 passing** |

**Clippy violations fixed (recurring patterns):**
- `clippy::unchecked_time_subtraction` (p1-c004): `Instant::now() - Duration` → `.checked_sub().unwrap()` in test fixtures
- `clippy::manual_let_else` (p1-c007): `match opt { Some(x) => x, None => return }` → `let Some(x) = opt else { return }`
- `clippy::match_same_arms` (p1-c007): duplicate 401 arms in `IntoResponse` → merged arm
- `clippy::double_must_use` (p1-c007): `#[must_use]` on fn returning `Router` (already `#[must_use]`)

---

## 4. Technical Debt Introduced

| Item | Severity | Remediation Phase |
|---|---|---|
| `frf-postgres-cdc` consumer loop body is dead code — not a working implementation | **HIGH** | Requires completing before Phase 1 exit criterion (end-to-end entity sync) is fully satisfied. `tokio-postgres 0.7` copy-both protocol must be hand-rolled or a capable crate found. See §7 gate question. |
| `PublishUseCase` performs JWT verification but no Keto authz check on write path | **HIGH** | A tenant can publish events without authorization. Per CLAUDE.md security constraints: *"Per-event RLS: Keto check(subject, 'view', object_id) before every fan-out delivery"* — the publish side should also enforce a write permission. Fix before production deployment or Phase 2 publish-path clients arrive. |
| Keto cache TTL is hardcoded (60s) in `KetoAuthzProvider` | LOW | Phase 7 — make configurable via `GatewayConfig`; add Keto webhook invalidation |
| `ws_subscribe` opens Iggy subscription before WS upgrade completes | LOW | Phase 7 — move broker subscribe into the WS upgrade callback to avoid holding a consumer slot during HTTP handshake timeout |
| No backpressure on `handle_socket` — slow WS client stalls Iggy consumer mpsc buffer | MEDIUM | Phase 7 — add per-socket send timeout; drop frames or close socket on overflow |
| All integration tests are `#[ignore]` with no automated infra | MEDIUM | Phase 7 or CI work — docker-compose stack needed for `cargo test -- --ignored` in CI |
| DashMap check-cache is process-local — multiple gateway instances will serve stale grants | MEDIUM | Phase 7 — Redis-backed shared cache or accept 60s TTL ceiling as deliberate policy |

---

## 5. Open Decisions (unchanged from Phase 1 entry)

| Decision | Must Be Made Before |
|---|---|
| CRDT engine: Loro vs automerge-rs | **Phase 3** |
| UniFFI / flutter_rust_bridge version | Phase 3 kickoff |
| Connect-ES version confirmed | **Phase 2 kickoff** |
| Tonic confirmed at 0.14.6 | ✅ Resolved in Phase 1 |

---

## 6. Lessons Captured

1. **`tokio-postgres 0.7` has no logical replication support.** The planned approach using a `pg_replicate` crate was blocked by non-existence at the expected version. `tokio-postgres` can connect to Postgres but does not expose the copy-both protocol needed for `START_REPLICATION`. The consumer skeleton was scaffolded with WAL decode types and unit tests but the actual consumer loop is dead code. For production CDC, this requires either: (a) hand-rolling the copy-both protocol over raw `tokio_postgres::Connection`, (b) evaluating `pg_replicate 0.2.x` if available, or (c) using a different approach (e.g., direct libpq bindings).

2. **`PortError` has no `Unauthorized` variant.** The enum has only `Transport`, `PermissionDenied`, `NotFound`, `Timeout`, `Serialization`, `Upstream`. All identity/authz failures map to `PermissionDenied`. Adding a new variant is a semver consideration on a `#[non_exhaustive]` public enum — do not add one without a version bump on `frf-ports`.

3. **Axum generic state with trait bounds requires explicit turbofish at route registration.** `routes::subscribe::ws_subscribe::<L, A, I>` is required in `build_router` because Axum cannot infer adapter types from the state type at the monomorphisation boundary. This is correct Axum 0.8 usage.

4. **DashMap TTL cache for Keto is process-local.** This is a deliberate Phase 1 simplification. Multi-instance deployments will serve stale grants for up to 60s. This is documented and must be addressed before Phase 7 hardening.

5. **Clippy `pedantic` enforces `let...else` over manual match-on-Option/Result returns.** The pattern `match opt { Some(x) => x, None => return ... }` always fails `clippy::manual_let_else`. Use `let Some(x) = opt else { ... }` from the first write.

6. **RSA test key must be a real 2048-bit PKCS8 PEM.** Using the jsonwebtoken test suite's key works reliably. A dummy or hand-written key causes subtle DER parsing failures in `jsonwebtoken 9.x`.

---

## 7. Phase Gate Question — Operator Decision Required

**CDC status is a gate-level question before advancing to Phase 2.**

The Phase 1 exit criterion (IMPLEMENTATION-PLAN §08) is:
> *"Supabase-like entity sync end-to-end with RLS; Rust client; cache invalidation working."*

The subscribe → Keto RLS check → WS fan-out path **works**. The Postgres → CDC → Iggy spine path **does not work** (consumer loop body is dead code). The end-to-end entity sync flow described in the phase objective requires CDC to be functional.

**Two valid operator choices:**

| Choice | Rationale | Consequence |
|---|---|---|
| **A. Advance to Phase 2 with CDC carried forward** | Phase 2 (generated SDKs) does not depend on CDC; CDC can be completed as a p2-c000 prerequisite fix or standalone task in Phase 2 | Phase 1 exit criterion is partially satisfied; Phase 2 kickoff must include CDC completion as a blocking task |
| **B. Complete CDC before advancing** | Keeps the phase gate contract clean; Phase 1 truly exits when the end-to-end flow works | Adds time; requires resolving the `tokio-postgres` copy-both limitation first |

**This reflection does not make this choice. It is an operator decision.**

---

## 8. Recommended Next Phase (conditional on §7)

**If operator chooses A:** Phase 2 — Generated SDKs + entity-management adapter. The first change in Phase 2 must be `p2-c000-cdc-completion` before any SDK work proceeds.

**If operator chooses B:** Complete CDC in Phase 1 before opening Phase 2.

**Phase 2 scope (IMPLEMENTATION-PLAN §08):**
> Go, C#, browser-TS (Connect-ES) generated from frozen proto; the `prometheus-entity-management` RealtimeAdapter on the TS SDK.
> Exit: four SDKs hitting one gateway; entity graph updates live in a React app.

**Prerequisites before Phase 2 kickoff (regardless of CDC choice):**

1. Confirm Connect-ES version — verify `@connectrpc/connect` current release and `@connectrpc/connect-web` compatibility with frozen `proto-v1`.
2. Confirm protoc/buf toolchain version for Go + C# generation.
3. Set up `sdks/go/`, `sdks/ts/`, `sdks/csharp/` directory stubs with Dagger pipeline skeletons.
4. Verify `frf-gateway` tonic service registration is ready for Connect protocol layer.
5. Address `PublishUseCase` missing authz check (HIGH debt item §4) before Phase 2 publish-path clients arrive.

**Pre-Phase 2 assess command:**
```
/kbd-assess phase-2-generated-sdks
```
