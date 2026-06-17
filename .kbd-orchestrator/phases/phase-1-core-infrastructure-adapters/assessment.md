# Assessment: Phase 1 — Core Infrastructure Adapters

> **RFC-FRF-002 · Prometheus AGS**
> Status: assessment complete · 2026-06-17
> Prepared by: kbd-assess workflow

---

## 1. Objective

Wire the first real infrastructure behind the Phase 0 port seams. After Phase 1, the
system must be capable of an end-to-end flow: a Postgres row change → CDC capture →
Iggy spine publish → Keto RLS check → WebSocket fan-out to a verified subscriber.
This is the "Supabase Realtime with durability and ReBAC" demonstration that validates
the entire hexagonal architecture.

---

## 2. Current Codebase State

### What exists (Phase 0 output)

| Crate | Status | Notes |
|---|---|---|
| `frf-domain` | ✅ Complete | 7 newtype IDs, full envelope/entity/agent/sync/presence/signal types; 10 serde roundtrip tests pass |
| `frf-ports` | ✅ Complete | 6 port traits (LogBroker, AuthzProvider, IdentityVerifier, CrdtStore, MediaSignaler, FederationBridge); `PortError`; zero implementations |
| `frf-proto` | ✅ Complete | 6 proto files frozen at `proto-v1`; `prost-build` codegen; `frf_proto::fv1::*` types accessible |
| `frf-gateway` | ✅ Stub | Axum 0.8.8; `/healthz` 200 OK; WS echo; `build_router()` exported; **no adapter wiring, no subscription mux, no auth** |
| `frf-broker-iggy` | ❌ Missing | No crate; `LogBroker` trait defined but has zero implementations |
| `frf-authz-keto` | ❌ Missing | No crate; `AuthzProvider` trait defined but has zero implementations |
| `frf-identity-ory` | ❌ Missing | No crate; `IdentityVerifier` trait defined but has zero implementations |
| `frf-postgres-cdc` | ❌ Missing | No crate; not defined in workspace |
| `frf-app` | ❌ Missing | No crate; subscribe/publish use-cases not yet modeled |

### Workspace members

```toml
members = ["crates/frf-domain", "crates/frf-ports", "crates/frf-proto", "crates/frf-gateway"]
```

Four crates in workspace; six Phase 1 crates need to be added.

---

## 3. Version Currency Audit

| Dependency | Pinned in Workspace | Current (June 2026) | Action |
|---|---|---|---|
| `tonic` | `0.14` | `0.14.5` — still current; tonic joined CNCF grpc project; in maintenance mode; no 0.15 yet | **CONFIRMED — pin stays** |
| `tonic-build` | not in workspace | `0.14.5` (tracks tonic) | Add to `[workspace.dependencies]` |
| `iggy` (GQAdonis fork) | `git = "https://github.com/GQAdonis/iggy", branch = "master"` | Apache upstream is `iggy v0.9.0`; GQAdonis fork likely tracking upstream | **VERIFY fork is current at Phase 1 kickoff**; IggyClient API confirmed async with producer/consumer pattern |
| `axum` | `0.8.8` | `0.8.9` in Cargo.lock | Minor patch; no action needed |
| `@connectrpc/connect` | not yet | `2.1.2` | Phase 2 dependency; record for planning |
| Connect-ES browser transport | not yet | `@connectrpc/connect-web` tracks `@connectrpc/connect` | Phase 2 dependency |

### Iggy API surface (confirmed from upstream v0.9.0)

```rust
// Construction
let client = IggyClient::from_connection_string("iggy://user:secret@localhost:8090")
    .expect("valid connection string");

// Producer pattern
let producer = client
    .producer("stream_name", "topic_name")?
    .with_partitioning(Partitioning::balanced())
    .build();
producer.init().await?;
producer.send(vec![Message::from_str("payload")?]).await?;

// Consumer pattern (consumer group)
let consumer = client
    .consumer("consumer_id", "stream_name", "topic_name", partition_id)?
    .with_auto_commit(AutoCommit::After(AutoCommitAfter::PollingMessages))
    .build();
consumer.init().await?;
let messages = consumer.next().await?;
```

**Key design note:** Iggy uses streams (namespace) + topics (channel) + partitions. The
`LogBroker.ensure_channel()` method must create a stream/topic pair. The `LogBroker`'s
`Channel` maps to: Iggy stream = `tenant_id` scope, Iggy topic = `channel.path`.

---

## 4. Ory Stack Architecture (Phase 1 Scope)

### Identity flow (Kratos + Oathkeeper)

```
client request
  │
  ▼
Oathkeeper /decisions/<path>   ← JWT authenticator (JWKS url)
  │  validates Bearer token against Kratos-issued JWKS
  │  sets X-User-ID, X-User-Email (header mutator)
  │  re-mints gateway-internal JWT (id_token mutator)
  ▼
frf-gateway (Axum handler)
  │  receives re-minted token in Authorization: Bearer
  │  calls IdentityVerifier::verify(token) → VerifiedClaims
  ▼
VerifiedClaims { session_id, tenant_id, subject, email, roles }
```

**Implementation approach for `frf-identity-ory`:**
The `IdentityVerifier::verify()` implementation has two options:
1. **Oathkeeper-proxied** (preferred for production): trust Oathkeeper's re-minted JWT,
   verify signature against Oathkeeper's own JWKS (`/.well-known/jwks.json` on Oathkeeper API port).
   Near-zero latency at the gateway — Oathkeeper already verified upstream.
2. **Direct Kratos SDK** (useful for testing without Oathkeeper): call Kratos
   `GET /sessions/whoami` with the session cookie/token.

Phase 1 implements option 1 (Oathkeeper-proxied) with a JWKS-verifying adapter using
`jsonwebtoken` (Rust) or `ory-client` generated from the Ory OpenAPI spec.

### AuthZ flow (Keto)

```
at subscribe time (coarse check):
  POST /relation-tuples/check  { namespace, object: channel_path, relation: "subscribe", subject_id }
  → allow/deny → cache result keyed by (subject, relation, object)

at fan-out (per-event RLS):
  POST /relation-tuples/batch/check  [{ object: event.id, relation: "view", subject_id }, ...]
  → filter events where check = true

on grant/revoke:
  PUT  /relation-tuples  { namespace, object, relation, subject_id }  ← write tuple
  DELETE /relation-tuples?...                                          ← delete tuple
  → invalidate cache entries matching (*, relation, object)
```

**Critical scaling constraint (from IMPLEMENTATION-PLAN §07):**
Per-event, per-subscriber Keto checks naively add `O(subscribers × events)` HTTP calls.
Phase 1 must design the mitigation:
1. **Subscribe-time scoping**: cache `check(subject, "subscribe", channel)` result per
   subscriber. Fan-out only to subscribers whose cached check passed.
2. **Batch check on delivery**: use `POST /relation-tuples/batch/check` for objects with
   fine-grained RLS (entity IDs), not per-call single-check.
3. **Cache invalidation**: `DashMap<(subject, relation, object), bool>` in the gateway
   process; invalidated on tuple-delete webhook from Keto (Phase 7 hardening) or
   short TTL (Phase 1 pragmatic approach: 60s TTL).

**No Rust Keto client crate** — implement as `reqwest`-based HTTP adapter with typed
request/response structs derived from the Keto REST API.

---

## 5. Gap Analysis — Phase 1 Crates

### Gap 1: `frf-broker-iggy` — LogBroker implementation

**Missing entirely.** The `LogBroker` port has five methods:
- `publish(envelope) → Offset` — serialize `EventEnvelope` to JSON bytes, call Iggy producer
- `subscribe(channel_id, consumer_id, from) → EventStream` — Iggy consumer group, stream-as-futures
- `seek(cursor)` — Iggy `store_consumer_offset` call
- `ack(channel_id, consumer_id, offset)` — Iggy `store_consumer_offset`
- `ensure_channel(channel)` — Iggy `create_stream` + `create_topic` if not exists

**Design challenge:** `EventStream` is `Pin<Box<dyn Stream<Item = Result<EventEnvelope, PortError>> + Send>>`.
The Iggy consumer does not natively return a `Stream`; it is a poll-based API.
`frf-broker-iggy` must wrap the consumer in an `async_stream::stream!` macro or manual
`Stream` implementation polling Iggy's `consumer.next()` in a loop.

**Iggy channel mapping:**
- Iggy `stream_name` → `tenant_id.to_string()` (one stream per tenant for isolation)
- Iggy `topic_name` → `channel.path` (e.g., `"entity/user/updates"`)
- Iggy `partition_id` → consistent hash of `consumer_id` for rebalancing

**Dependencies needed:**
```toml
iggy = { workspace = true }
async-stream = "0.3"
futures-util = "0.3"
tokio-stream = { workspace = true }
```

### Gap 2: `frf-authz-keto` — AuthzProvider implementation

**Missing entirely.** Three methods map directly to Keto REST API calls:
- `check(tuple) → bool` — `POST /relation-tuples/check`
- `write(tuple)` — `PUT /relation-tuples`
- `delete(tuple)` — `DELETE /relation-tuples`

**Internal cache layer** (`DashMap`) for subscribe-time check results to avoid per-event
HTTP overhead. TTL-based invalidation in Phase 1; webhook-based in Phase 7.

**Dependencies needed:**
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
dashmap = { workspace = true }
serde_json = { workspace = true }
```

### Gap 3: `frf-identity-ory` — IdentityVerifier implementation

**Missing entirely.** One method: `verify(token: &str) → VerifiedClaims`.

**Implementation:** Verify Oathkeeper-re-minted JWT using `jsonwebtoken` crate against
a cached JWKS fetched from Oathkeeper's `/.well-known/jwks.json` endpoint. JWKS is
refreshed on key-not-found (key rotation). Extract `sub`, `email`, `tenant_id`, `roles`
from claims.

**Dependencies needed:**
```toml
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio = { workspace = true }
```

### Gap 4: `frf-app` — Subscribe/publish use-cases

**Missing entirely.** The IMPLEMENTATION-PLAN §02 describes `frf-app` as the
application layer orchestrating ports. Phase 1 needs the minimal application logic to
wire the subscription pipeline:

```rust
pub struct SubscribeRequest { pub channel_id: ChannelId, pub token: String, pub from: Offset }
pub struct SubscribePipeline<L: LogBroker, A: AuthzProvider, I: IdentityVerifier> { ... }

impl<L, A, I> SubscribePipeline<L, A, I> {
    pub async fn execute(&self, req: SubscribeRequest) -> Result<EventStream, AppError>;
    // 1. verify token → VerifiedClaims
    // 2. check(subject, "subscribe", channel) → allow/deny
    // 3. broker.subscribe(channel_id, consumer_id, from) → raw stream
    // 4. filter stream: per-event check(subject, "view", event.id) using cached results
    // 5. return filtered EventStream
}
```

**`frf-app` must NOT import any adapter crate** — only `frf-domain` + `frf-ports`.

### Gap 5: `frf-postgres-cdc` — WAL logical replication consumer

**Missing entirely.** Produces `EventEnvelope` facts onto the Iggy spine from Postgres WAL.

**Implementation approach:**
- Use `pg_replicate` crate (maintained Rust logical replication client) or `tokio-postgres`
  with `pgoutput` logical replication plugin.
- Decode `INSERT`/`UPDATE`/`DELETE` WAL events → `EntityChange` domain types → `EventEnvelope`.
- Publish to spine via `LogBroker.publish()`.
- Heartbeat + LSN checkpointing so restarts resume from last confirmed LSN.

**Note:** `pg_replicate` crate is the recommended approach — it abstracts the
`START_REPLICATION SLOT` command and WAL message decoding. Verify it is still maintained
before adding.

**Dependencies needed:**
```toml
pg_replicate = "0.2"   # verify current version
tokio-postgres = { version = "0.7", features = ["with-serde_json-1"] }
```

### Gap 6: Gateway subscription mux

**Gateway has no subscription wiring.** `frf-gateway/src/lib.rs` exports only `build_router()` with `/healthz` and `/ws` echo. Phase 1 must add:

1. **WebSocket subscription handler** at `/ws/v1/subscribe` that:
   - Accepts `Authorization: Bearer <token>` header + `channel` query param
   - Calls `SubscribePipeline::execute()`
   - Streams `EventEnvelope` JSON frames to the WebSocket client
   - Handles client disconnect cleanly (no panic, no stuck subscription)

2. **State injection** — Axum `State<Arc<AppState>>` carrying `LogBroker`, `AuthzProvider`,
   `IdentityVerifier` trait objects.

3. **Publish endpoint** — `POST /v1/publish` for server-side event injection.

**No gRPC service registration yet** (Phase 2 with Connect-ES). The WS mux is Phase 1's
delivery target.

---

## 6. `frf-app` — Application Layer Design

This is the missing layer between ports and the gateway. Following the IMPLEMENTATION-PLAN §02
dependency rule:

```
frf-domain       (Layer 0 — types)
frf-ports        (Layer 1 — trait seams)
frf-app          (Layer 1 — use-cases, imports domain + ports only)
frf-broker-iggy  (Layer 2 — adapter, imports domain + ports)
frf-authz-keto   (Layer 2 — adapter, imports domain + ports)
frf-identity-ory (Layer 2 — adapter, imports domain + ports)
frf-gateway      (Interface — imports app + adapters, wires everything)
```

`frf-app` provides two core use-cases for Phase 1:
- `SubscribePipeline` — verify → authz check → subscribe → RLS-filtered stream
- `PublishUseCase` — validate → authz write-check → broker.publish()

Both are generic over the port traits, testable with `mockall` mocks, no real network.

---

## 7. Risk Register — Phase 1 Specific

| Risk | Severity | Mitigation |
|---|---|---|
| Iggy GQAdonis fork API drift from Apache upstream | HIGH | Audit fork diff at kickoff; if fork is stale, submit PR upstream or pin to a known-good commit |
| Per-event Keto check latency at fan-out scale | HIGH | Subscribe-time cache (DashMap, 60s TTL) designed in Phase 1; batch check API for RLS |
| `async_stream::stream!` in IggyBroker polling loop: backpressure | MEDIUM | Use `tokio_stream::wrappers::ReceiverStream` with bounded `mpsc::channel` as buffer; drop oldest on full |
| `pg_replicate` crate maintenance status | MEDIUM | Verify before adding; fallback is `tokio-postgres` + raw `pgoutput` decode |
| Oathkeeper running in dev environment | MEDIUM | Provide `MockIdentityVerifier` (always-pass) for local dev without Oathkeeper; injected via feature flag |
| `jsonwebtoken` JWKS verification: key rotation race | LOW | Cache JWKS; retry with refresh on `InvalidKeyFormat` / `InvalidSignature` — standard rotation pattern |

---

## 8. Workspace Changes Required

Add to `Cargo.toml` `[workspace.members]`:
- `crates/frf-app`
- `crates/frf-broker-iggy`
- `crates/frf-authz-keto`
- `crates/frf-identity-ory`
- `crates/frf-postgres-cdc`

Add to `[workspace.dependencies]`:
```toml
tonic-build        = "0.14"
async-stream       = "0.3"
futures-util       = "0.3"
reqwest            = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
jsonwebtoken       = "9"
tokio-postgres     = { version = "0.7", features = ["with-serde_json-1"] }
async-trait        = "0.1"   # already in frf-ports but not workspace-pinned
```

---

## 9. Exit Criteria for Phase 1

| Criterion | Verification |
|---|---|
| `frf-broker-iggy` compiles and unit tests pass (mock Iggy server or real) | `cargo test -p frf-broker-iggy` |
| `frf-authz-keto` compiles; `check/write/delete` tested against mock HTTP server | `cargo test -p frf-authz-keto` (wiremock or httpmock) |
| `frf-identity-ory` compiles; JWKS verify tested with a known test JWT | `cargo test -p frf-identity-ory` |
| `frf-app` SubscribePipeline tested with mockall mocks — no real network | `cargo test -p frf-app` |
| `frf-postgres-cdc` compiles and decodes a test WAL event to `EntityChange` | `cargo test -p frf-postgres-cdc` |
| `frf-gateway` WS subscription endpoint returns events to a test WebSocket client | `cargo test -p frf-gateway` (axum-test + tokio::spawn) |
| Full workspace: `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` | 0 warnings |
| Full workspace: `cargo test --workspace` | all pass |
| Integration smoke test: publish → subscribe → receive (local Iggy + stub authz) | documented test |

---

## 10. Next Step

```
/kbd-plan phase-1-core-infrastructure-adapters
```
