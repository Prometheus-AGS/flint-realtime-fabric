# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project is pre-1.0 and
does not yet follow strict SemVer for the workspace (the frozen `proto-v1` contract is
the stable interface).

## [Unreleased]

### Phase 17 — plane completion & release audit

Independently re-audited the release claim, wired the QA gate, and completed the
pure-Rust deferred planes.

#### Verification & process

- Independent production-readiness re-audit against real code: **zero CRITICAL, zero
  HIGH** survive in the production build; `cargo check` / `clippy` (pedantic +
  `unwrap_used`) / `fmt` green.
- Removed the committed `compose.override.yml` (auto-merged dev-secret + auth-bypass
  footgun); ship `compose.override.example.yml` with placeholders instead.
- `JWT_ISSUER` is now **mandatory in production** — a release build fails to boot without
  it (dev builds keep the warning).
- Per-change QA gate wired into execute (`.kbd-orchestrator/constraints.md` +
  `qa-gate.sh`), closing the phase-16 process gap where QA never ran.

#### Deliverables

- **`EntityService`** gateway server (read/watch, auth-guarded: identity + tenant-equality
  + Keto `view`).
- **`AuthzService`** gateway server (relation check/write/delete via Keto, tenant-scoped).
  **All six proto services now have server implementations.**
- `frf-sdk-rust` binds all five non-Spine service clients (incl. Entity/Authz); FFI mobile
  subscribe is now resilient (reconnect/replay) and exposes `ack`; TS/Go/C# wrappers add
  Entity/Authz.
- `frf-cli`: `broker offsets` (read stored consumer offset) and `cdc slot create|drop`
  (manage the replication slot) — the CLI now matches its advertised surface.
- **Dart SDK** bindings generated via `uniffi-bindgen-dart` and committed (CRDT surface
  usable; async transport surface blocked by a generator bug — documented in
  `sdks/dart/GENERATED.md`, not hidden).

#### Deferred to a future phase (documented, not silently missing)

- str0m sovereign SFU real WebRTC (still signaling-only, gated off); Matrix inbound /
  ATProto outbound / LiveKit cross-node inbound relay; admin-ui interactive OIDC login;
  Dart async-transport bindings (pending an upstream `uniffi-bindgen-dart` fix).

### Phase 16 — production hardening

Closes the release-blocking gaps found in the production-readiness audit.

#### Security

- Production `compose.yml` no longer compiles in or activates the dev auth bypass; the
  release image is built without the `dev-endpoints` feature, so `DEV_NO_AUTH` cannot
  take effect there.
- App-layer tenant-equality guard on publish (JWT tenant must match the channel tenant).
- JWT issuer (`iss`) verification via `JWT_ISSUER`.
- `clippy::unwrap_used` / `expect_used` enforced workspace-wide in CI.
- Rate-limiting, request body-size limit, and CORS middleware on the gateway.
- flint-gate signing secret externalized out of the repo.
- Cedar surfaces policy-evaluation errors instead of silently denying.

#### Deliverables

- **`frf-sdk-rust`** — hand-written Rust client with reconnection/backoff and
  replay-from-offset; reconnection surfaced in the TS/Go/C# SDKs.
- **`frf-cli`** (`frf`) — operator CLI: Keto tuple seed/revoke, broker checkpoint,
  CDC status.
- FFI transport (connect/auth/publish/subscribe) for Swift and Kotlin via UniFFI.
- Gateway registers Spine, Signal, Sync, and Agent gRPC services (gRPC-web enabled);
  Sync/Agent/Signal clients bound in the SDKs.
- All SDKs generate from the frozen `proto-v1` (C# proto fork removed).

#### Operability

- `/readyz` readiness probe (Keto/JWKS/Iggy); `/metrics` (Prometheus); graceful
  shutdown on SIGTERM/SIGINT (drains in-flight requests + WS streams).
- Semantic config validation at boot (fails fast on invalid config).
- Keto schema-migration step in compose.

#### Docs

- `docs/ENVIRONMENT.md` (env-var reference), `docs/RUNBOOK.md` (operations),
  `docs/SECURITY.md` (security model), `.env.example`.
- `LICENSE` (MIT), `CONTRIBUTING.md`, `SECURITY.md` (disclosure policy), this changelog,
  and `docs/decisions/adr-003-ffi-codegen-versions.md`.

#### Deferred at the time (see Phase 17 for what shipped since)

- str0m sovereign SFU (real WebRTC); Matrix/ATProto federation protocol impls;
  `EntityService`/`AuthzService` gateway servers; Dart transport bindings; LiveKit
  cross-node inbound relay. Phase 17 delivered the `EntityService`/`AuthzService` servers
  and the Dart CRDT bindings; the rest remain deferred.

### Earlier phases (0–15)

Foundations through live Layer-3 E2E validation — see
`.kbd-orchestrator/phases/` for per-phase plans, assessments, and reflections, and
`docs/IMPLEMENTATION-PLAN.md` (RFC-FRF-002) for the phase-by-phase build plan.
