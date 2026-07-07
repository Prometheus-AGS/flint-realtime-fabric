# Production-Readiness Assessment — flint-realtime-fabric

> Authored: 2026-07-06
> Assessor: kbd-assess (multi-agent audit — 9 dimensions × adversarial verification)
> Scope: FULL production release readiness (docs, SDKs, CLIs, utilities, admin UI,
> security holes, logic holes / abstractions that don't connect)
> Method: 9 parallel dimension auditors → 61 findings → each finding adversarially
> re-verified by an independent skeptic agent (74 agents total, 0 errors).
> 40 findings CONFIRMED as-stated; 21 severity-corrected on verification.

---

## VERDICT

**NOT PRODUCTION-READY.** This is an advanced Phase-15 prototype, not a v1.

The workspace compiles clean (`cargo check --workspace` = exit 0), `proto-v1` is
frozen, MSRV is pinned (1.85), and the **happy-path data plane is genuinely wired**
(JWT-verified publish/subscribe over Iggy → Keto authz → per-event `view` filter →
CDC ingest). That core is real, not stubbed.

But a release is blocked by independent **CRITICAL** defects in three categories:

1. **Security-config hole** — the repo's production-shaped `compose.yml` compiles
   in *and activates* a total JWT/Cedar auth bypass (`CARGO_FEATURES: dev-endpoints`
   + `DEV_NO_AUTH: "true"`).
2. **Advertised-but-non-functional planes** — the "sovereign" str0m SFU never
   imports str0m and moves no media; Matrix inbound is `stream::empty()`; the entire
   admin UI cannot reach the gateway (no Connect/gRPC-web endpoint, no `/ws/v1/signal`).
3. **Missing shipped deliverables** — `frf-sdk-rust` and `frf-cli` **do not exist**;
   three mobile SDKs are CRDT-only or non-compiling stubs; no SDK implements
   reconnection; 5 of 6 proto services have no client surface.

Layered on top: HIGH operational gaps (static `/healthz`, no readiness probe, no
rate-limit/CORS, no graceful shutdown, no ops/deploy/security docs) and a
documented-but-unenforced `clippy::unwrap_used` gate.

**Severity distribution (post-verification):** 7 CRITICAL · 15 HIGH · 24 MEDIUM · 15 LOW.

---

## SINGLE MOST IMPORTANT FIX

**Remove the auth bypass from `compose.yml` (C1).** Move `CARGO_FEATURES: dev-endpoints`
and `DEV_NO_AUTH: "true"` into a clearly-named `compose.dev.yml` / override only, and
make `dev_no_auth()` unreachable in any release artifact. It is simultaneously (a) a
complete collapse of auth + tenant isolation, (b) reachable by the obvious "lift the
compose file to prod" path, and (c) a trivial config change. Every other finding
assumes the auth boundary holds. Then enforce that boundary in code: turn on
`clippy::unwrap_used` in CI (H12/#22), add the app-layer tenant-equality check (H2/#8),
and ship a real readiness probe (H8/#16).

---

## RELEASE-BLOCKING — CRITICAL (7)

| # | Finding | Location | Concrete failure |
|---|---------|----------|------------------|
| C1 | Production-shaped `compose.yml` compiles + activates total auth bypass | `compose.yml:9,29`; honored at `routes/publish.rs:55`, `routes/subscribe.rs:54` | Any deploy using `compose.yml` as a prod template accepts empty tokens on `/v1/publish` and `/ws/v1/subscribe` → forged cross-tenant writes, unauthenticated reads. |
| C2 | Admin UI cannot reach gateway — no Connect/gRPC-web endpoint for SpineService | `admin-ui/.../gateway.ts:8`; gateway `lib.rs:63` (no Spine route), `main.rs:347` (only AgentService, no `tonic_web`) | Default landing page (Entities) issues `SpineService/Subscribe` → 404/405; flagship admin surface non-functional against a real gateway. |
| C3 | Signaling UI connects to `/ws/v1/signal` — route does not exist | `admin-ui/.../signalingService.ts:24`; router `lib.rs:63` has only dev-gated `/dev/inject-signal` | WebRTC signaling admin screen dead against any real build. |
| C4 | str0m "sovereign SFU" is a fake — str0m crate never imported, no WebRTC | `crates/frf-media-str0m/src/sfu.rs:34`; wired at `main.rs:311` | With `SFU_MODE=sovereign` (compose default): SDP offers route as JSON, no peer connection, no answer, no RTP. Media never flows while plane reports healthy. |
| C5 | `frf-sdk-rust` crate missing — canonical home for reconnection/lifecycle | `crates/frf-sdk-rust` (absent) | The one hand-written client all business/CRDT/reconnect logic should route through doesn't exist; no reference for other SDKs to inherit. |
| C6a | Swift/Kotlin FFI SDKs expose only 3 CRDT byte functions — no transport | `crates/frf-ffi/src/lib.rs:15` | Mobile apps can merge CRDT blobs but cannot connect/auth/subscribe/publish. |
| C6b | Dart SDK generated bridge dir is empty (`.gitkeep`) — does not compile | `sdks/dart/lib/frf_dart.dart:7` | Dart package fails `pub get`; no usable client. |

---

## RELEASE-BLOCKING — HIGH (15)

| # | Finding | Location |
|---|---------|----------|
| H1 | No app-layer tenant-equality assertion on publish/subscribe (JWT tenant vs channel tenant) | `frf-app/src/publish.rs:49`, `subscribe.rs:51` |
| H2 | No rate limiting, no CORS on any gateway route (`tower-http` declared, never wired) | `frf-gateway/src/lib.rs:63` |
| H3 | Cedar = authorize-everything: `Entities::empty()` + blanket permit-all default policy | `frf-policy-cedar/src/lib.rs:91` |
| H4 | LiveKit hosted SFU never listens to server — inbound signals lost cross-node | `frf-media-livekit/src/adapter.rs:108` |
| H5 | Federation bridges stamp random per-boot `TenantId::new()`/`ChannelId::new()` — federated events land where no JWT matches | `frf-gateway/src/main.rs:192,199` |
| H6 | No SDK (any language) implements reconnection/backoff/retry | `sdks/ts/src/client.ts:27` |
| H7 | Only SpineService is bound — 5 of 6 proto services have no client surface | `sdks/{ts,go,csharp}` |
| H8 | `/healthz` is a static stub; no `/readyz` readiness probe exists | `frf-gateway/src/routes/health.rs:4` |
| H9 | Gateway does not embed/serve admin-ui (no `rust-embed`/`ServeDir`) | `frf-gateway/src/lib.rs:63` |
| H10 | Admin UI has no login/auth flow — every call carries an empty JWT | `admin-ui/.../authStore.ts:10` |
| H11 | Default UI gateway URL (`localhost:4000`) ≠ gateway bind (`0.0.0.0:8080`) | `admin-ui/.../gateway.ts:5` |
| H12 | `clippy::unwrap_used` hard gate has ZERO enforcement (`pedantic` ≠ restriction group) | `crates/*/Cargo.toml`, `ci.yml:38` |
| H13 | `frf-cli` missing — no first-party operator tooling (seed Keto, manage CDC slots, checkpoint) | `crates/frf-cli` (absent) |
| H14 | No env-var reference — ~30 gateway env vars incl. secrets undocumented; no `.env.example` | `docs/DEVELOPMENT.md:71` |
| H15 | No deployment/operations runbook (prod topology, secrets, CDC-slot lifecycle, scaling) | `docs/` |

---

## PER-AREA STATUS

| Area | Status | Reason |
|------|--------|--------|
| **Security / AuthZ** | PARTIAL (blocking) | JWT verify + Keto are real and wired, and per-event `view` check IS enforced (`subscribe.rs:81` — a strength). But `compose.yml` ships a total bypass, no app-layer tenant-equality check, Cedar is allow-all/deny-all, JWT `iss` unvalidated, no rate-limit/CORS. |
| **Wiring / Logic-holes** | STUB (blocking) | Sovereign str0m SFU has no WebRTC; Matrix inbound `stream::empty()`; LiveKit never listens cross-node; ATProto outbound `Err`s; federation uses random tenant IDs. Multiple advertised planes non-functional. |
| **Error-handling** | PARTIAL | `unwrap_used` gate unenforced; WASM merge swallows peer-delta errors; redb `entry.ok()?` swallows read errors; lenient `jti` fabrication. Fail-fast on missing secrets is good. |
| **CRDT / Store** | PARTIAL | Loro merge lives in one place (good; ADR-001 resolved). But redb composite-key upper-bound increments the wrong segment (data-loss edge), SurrealDB never creates its documented UNIQUE index, and the Surreal store isn't even wired (`InMemoryCrdtStore` only). |
| **SDKs** | MISSING / STUB (blocking) | `frf-sdk-rust` absent; Swift/Kotlin CRDT-only; Dart non-compiling; no reconnection anywhere; only SpineService bound; C# forks a stale proto copy. TS (508 files) and Go (generated pb+connect) are the only real ones. |
| **CLI / Ops** | MISSING (blocking) | `frf-cli` absent; static `/healthz`, no `/readyz`, no `/metrics`; no graceful shutdown; no Keto/CDC migration step; config validates presence not semantics. |
| **Admin-UI** | STUB (blocking) | Does not connect to gateway at all (no Connect endpoint, no `/ws/v1/signal`, URL mismatch, no login, not embedded). Clean code otherwise (0 `any` types, feature architecture followed). E2E asserts only static rendering. |
| **Docs** | PARTIAL | No LICENSE file (MIT declared), no SECURITY/CONTRIBUTING/CHANGELOG, no env reference, no deploy runbook, no proto API reference; README documents non-existent crates and is ~3 phases stale. Existing content is accurate. |
| **Quality-gates** | PARTIAL | `unwrap_used` documented-only; `#[non_exhaustive]` missing on `SfuMode`/`PolicyEngineMode`; `deny(warnings)` inconsistent across 5 crates; GitHub vs Dagger clippy invocations diverge. Strengths: MSRV pinned, 31/32 enums non-exhaustive, no file >500 lines. |

---

## CROSS-TOOL PROGRESS

- `p15-c001-dagger-port-fix`: COMPLETE (Stage 10 healthz poll + GATEWAY_URL → port 28080).
- Active phase-15 work (Stage 10 live run) is orthogonal to this readiness audit and remains `operator_action_required` (requires DinD).

---

## NOTES ON VERIFICATION (anti-sycophancy)

Adversarial verification **corrected 21 of 61 findings** — this report is not
rubber-stamped:

- The Keto cache "cross-tenant poisoning" claim was downgraded **CRITICAL → MEDIUM**:
  all `object` values are globally-unique v4 UUIDs, so a cached decision can only ever
  serve the one real object — no exploitable leak today (but a latent isolation gap if
  object ids ever become tenant-relative). (#23)
- The Matrix inbound stub (#10) and ATProto outbound `Err` (#27) were noted as having
  no live production caller today, so held below CRITICAL.
- Finding #48 (hardcoded flint-gate signing secret in `compose.yml:85`) was marked
  OVERSTATED as a finding but remains a **real secret-in-repo hygiene issue** — rotate
  and externalize before any public release.

Strengths independently confirmed and worth preserving: clean compile, frozen proto,
pinned MSRV, per-event authz filter, no oversized files, admin-ui type discipline,
fail-fast secret loading, single-home CRDT merge.

---

## RECOMMENDED NEXT PHASE

**phase-16-production-hardening** (or split into security + completeness tracks):

1. **Security gate (do first):** C1 compose bypass; H1 tenant-equality; H12 enforce
   `unwrap_used`; JWT `iss` validation; rate-limit/CORS; externalize the signing secret.
2. **Make advertised planes real or remove them:** str0m SFU (C4), Matrix/ATProto/LiveKit
   federation + signaling wiring, admin-UI ↔ gateway transport (C2/C3/H9/H10/H11).
3. **Ship the missing deliverables:** `frf-sdk-rust` (C5) with reconnection, then FFI
   transport for Swift/Kotlin/Dart (C6), `frf-cli` (H13).
4. **Operability:** `/readyz`, `/metrics`, graceful shutdown, Keto/CDC migration step.
5. **Docs:** env reference + `.env.example`, deploy runbook, security model, LICENSE,
   fix stale README.

Full verified finding table (all 61) in the accompanying Artifact and
`assessment-production-readiness-findings.json`.

ASSESSMENT COMPLETE
