# Goals — phase-16-production-hardening

> Seeded from: phase-15 production-readiness assessment (2026-07-06)
> Source: `.kbd-orchestrator/phases/phase-15-stage10-live-run-and-retries/assessment-production-readiness.md`
> Verdict feeding this phase: **NOT production-ready** — 7 CRITICAL, 15 HIGH, 24 MEDIUM, 15 LOW.

The audit found the happy-path data plane is genuinely wired, but security config,
advertised media/federation planes, SDKs, CLI, and the admin-UI transport are not
release-ready. This phase closes the release-blocking gaps in dependency order:
**security boundary first**, then make advertised planes real (or remove them), then
ship missing deliverables, then operability and docs.

The goals are ordered as tracks. Do not advance a later track while an earlier CRITICAL
remains open. Each goal cites the assessment finding IDs it closes.

---

## G1 — Close the security boundary (do first, blocks everything)

The whole system's premise is sovereign auth + tenant isolation. Every other finding
assumes this boundary holds; it currently does not.

- **G1.1 (C1)** Remove the auth bypass from the production-shaped `compose.yml`
  (`CARGO_FEATURES: dev-endpoints` + `DEV_NO_AUTH: "true"`). Move both to a clearly
  named `compose.dev.yml` / override only. Make `dev_no_auth()` unreachable in any
  release artifact — gate at the type level, not by remembering an env var.
- **G1.2 (H1)** Add an app-layer tenant-equality assertion: on publish and subscribe,
  reject when the verified JWT `tenant_id` ≠ the target channel/envelope `tenant_id`
  (`frf-app/src/publish.rs`, `subscribe.rs`). Defense in depth beneath Keto.
- **G1.3 (H12)** Enforce the `clippy::unwrap_used` gate in CI (it is documented but
  `pedantic` does not include the restriction group). Turn it on, then eliminate the
  real library unwraps concentrated in `frf-crdt` and `frf-store-redb`.
- **G1.4 (#25)** Verify JWT issuer (`iss`) in `frf-identity-ory`.
- **G1.5 (H2)** Wire `tower-http` rate limiting, a body-size limit, and an explicit
  CORS policy on the gateway router.
- **G1.6 (#48)** Rotate and externalize the flint-gate signing secret currently
  committed in `compose.yml`; load from env/secret manager.
- **G1.7 (H3)** Make Cedar functional or explicitly no-op: supply real `Entities` and
  a policy set that can actually match, or document Cedar as disabled — no silent
  allow-all / deny-all.

**Exit:** No release artifact can bypass auth. Tenant-equality enforced in code.
`unwrap_used` green in CI with zero library unwraps. iss verified. Rate-limit/CORS live.

---

## G2 — Make advertised planes real, or remove them

Several planes report healthy while doing nothing. Either implement them or delete the
advertisement so the surface matches reality.

- **G2.1 (C4)** str0m "sovereign SFU": implement real WebRTC (import str0m, negotiate
  Rtc/SDP/ICE, forward RTP) — or gate `SFU_MODE=sovereign` off by default and mark it
  unimplemented. Fix the `from_session`/`to_session` misroute.
- **G2.2 (C2/C3/H9/H10/H11)** Wire the admin-UI ↔ gateway transport end to end:
  serve a Connect/gRPC-web endpoint for SpineService (`tonic_web` + `accept_http1`),
  add the real `/ws/v1/signal` route, embed admin-ui in the gateway
  (`rust-embed`/`ServeDir`), add a login/auth flow so calls carry a real JWT, and
  reconcile the default gateway URL/port across UI and gateway.
- **G2.3 (H4)** LiveKit hosted mode: listen to the LiveKit server so inbound signals
  from remote peers/nodes arrive — or scope it to single-process and document the limit.
- **G2.4 (H5)** Federation bridges: replace per-boot random `TenantId::new()`/
  `ChannelId::new()` with configured tenant/channel identifiers so federated events
  land where a subscriber's JWT matches.
- **G2.5 (#10/#27)** Matrix inbound (`stream::empty()`) and ATProto outbound (`Err`):
  implement, or mark the bridge direction unsupported and stop wiring it as bidirectional.

**Exit:** Every plane the gateway boots either functions end to end or is honestly
labeled unimplemented/disabled — no "healthy but does nothing."

---

## G3 — Ship the missing deliverables

- **G3.1 (C5)** Create `frf-sdk-rust` — the hand-written Rust client that is the single
  home for connection lifecycle, reconnection/backoff, and CRDT merge.
- **G3.2 (C6)** Give the FFI surface real transport: extend `frf-ffi` beyond the 3
  CRDT byte functions so Swift/Kotlin/Dart can connect/auth/subscribe/publish. Fix the
  empty Dart bridge dir so the package compiles.
- **G3.3 (H6)** Implement reconnection/backoff/replay-from-offset once in `frf-sdk-rust`
  and surface it through TS/Go/C#; use the resumable-offset proto primitives.
- **G3.4 (H7)** Bind the remaining 5 proto services (sync, agent, signal, entity, authz)
  in the real SDKs, and register their servers in the gateway where missing.
- **G3.5 (H13)** Create `frf-cli` — seed Keto tuples, manage CDC replication slots,
  inspect broker offsets, force checkpoints. Surface `AuthzProvider::write()`.
- **G3.6 (#32)** Stop the C# SDK forking its own proto copy; generate from the frozen
  `proto-v1` source of truth.

**Exit:** `frf-sdk-rust` and `frf-cli` exist and are usable. All 6 proto services are
reachable through at least the Rust + TS SDKs. Reconnection works in one place.

---

## G4 — Operability

- **G4.1 (H8)** Real `/readyz` readiness probe that checks Iggy/Keto/JWKS liveness;
  keep `/healthz` as liveness. Ensure compose/K8s `depends_on` gates on readiness.
- **G4.2 (#34)** Add a `/metrics` endpoint (Prometheus) alongside tracing spans.
- **G4.3 (#35)** Graceful shutdown on SIGTERM — drain in-flight requests and WS streams.
- **G4.4 (#36)** Config validation of semantic validity, not just presence (e.g. hosted
  SFU must not boot with empty LiveKit credentials).
- **G4.5 (#37)** Add a Keto schema-migration step so persistent-DSN deployments start.

**Exit:** A dead dependency takes the pod out of rotation; SIGTERM drains cleanly;
misconfiguration fails fast at boot; gateway is observable via `/metrics`.

---

## G5 — Documentation & release artifacts

- **G5.1 (H14)** Env-var reference + `.env.example` documenting all ~30 gateway vars,
  which are secrets, and which are dev-only bypasses.
- **G5.2 (H15)** Deployment/operations runbook: prod topology, secret provisioning,
  CDC-slot lifecycle/recovery (stalled slot → WAL pinning → disk-fill), scaling,
  upgrade/rollback.
- **G5.3 (#41)** Security-model document (auth boundary, tenant isolation, Keto/Cedar
  roles).
- **G5.4 (#42/#43)** Fix the stale README: remove references to previously-missing
  crates once they exist, update the phase status.
- **G5.5 (#54/#57/#58/#59)** Add LICENSE file (MIT is declared), CONTRIBUTING,
  CHANGELOG, SECURITY.md, a proto-derived API reference, and capture the
  UniFFI/flutter_rust_bridge/Connect/tonic version decision in an ADR.

**Exit:** An SRE and an external contributor can each stand the system up and operate
it from the docs alone.

---

## Phase completion criteria

- All CRITICAL (7) and HIGH (15) findings from the phase-15 assessment are closed or
  explicitly, honestly deferred with rationale.
- A re-run of the production-readiness audit returns **no CRITICAL** and a
  materially reduced HIGH count.
- `cargo check --workspace`, `cargo clippy -- -D warnings -W clippy::pedantic
  -W clippy::unwrap_used`, and `cargo fmt --check` all pass in CI.

## Non-goals

- The Stage 10 live-run / DinD work (phase-15) is orthogonal and remains its own track.
- Net-new features beyond closing assessed gaps (YAGNI).
