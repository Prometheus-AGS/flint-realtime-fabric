# Feedback review — production readiness across projects

Date: 2026-09-15. Method: local source and artifact inspection, not a runtime audit.
The supplied assessment is input to planning; its percentages, PR descriptions,
historical test counts and build conclusions are not current release evidence.
Source revisions, dirty-file inventory and feedback hash are in `entry-state.json`.

## Corrections that affect the plan

| Feedback claim | Current evidence | Planning consequence |
|---|---|---|
| WatchEntity may be a differently named WatchEntityType | FRF `proto/flint/v1/entity.proto:44–51` accepts one entity ID. Forge `crates/fdb-realtime/src/lib.rs:183–205` still returns `StreamError::Unavailable` for table watches. | This is a missing capability, not a naming correction. Design and deliver the producer and consumer contract together. |
| Forge defaults to Fabric | Forge `crates/fdb-gateway/src/realtime_source.rs:21–33` defaults to LISTEN unless the exact value is `fabric`. | Preserve the working default until the new Fabric adapter passes deployment-specific integration proof. |
| Matrix inbound is absent because the gateway calls it a stub | FRF `crates/frf-gateway/src/federation.rs` logs that, but the composed `ReqwestMatrixClient::room_event_stream` in `crates/frf-bridge-matrix/src/client.rs:75` performs authenticated `/sync`, and `MatrixBridge::subscribe` delegates to it. | Fix contradictory descriptions and prove actual traffic/recovery. Do not spend a change reimplementing an existing inbound client. |
| Gate only signs HS256 | Gate `crates/flint-gate-core/src/auth/jwt_mint.rs` supports asymmetric signing, key IDs and replica grants; FRF `crates/frf-identity-ory/src/verifier.rs` verifies RS256. ADR-009 records a bounded real Gate-to-FRF proof. | Verify the chosen deployment's signing configuration, route and lifecycle; do not infer incompatibility from old harness comments. |
| A clean CI test run is the first release gate | `AGENTS.md` and `CLAUDE.md` prohibit all CI/CD test execution. `.github/workflows/ci.yml` still contains `cargo test --all`; Dagger contains Playwright execution stages. | Remove CI test paths and transfer required proofs to a locally composed stack. CI remains build/lint/typecheck/format/package only. |
| Lack of Dagger cargo tests is itself a readiness defect | The policy explicitly requires local integration evidence, independent of a particular orchestrator. Dagger stage 4 still attempts flutter_rust_bridge despite ADR-003's UniFFI decision. | Repair codegen and build reproducibility; do not add tests to a CI pipeline. |
| July signoff certifies current main | `docs/RELEASE-SIGNOFF.md` explicitly scopes itself to phases 16–17, dated 2026-07-07. Later work includes the p38 audit and current uncommitted shape changes. | Preserve the historical record and produce a new revision-bound release verdict. |
| The only important deployment discrepancy is SFU_MODE | SSR also selects `AUTHZ_BACKEND=verified-identity`, `CDC_ENABLED=false`, and `POLICY_ENGINE=none`; its authz adapter allows named relations without an object lookup. | Audit effective authorization and enabled data lanes before claiming general-purpose Keto-protected realtime. |
| PEM has no real-adapter test / its live test merely skips | PEM `packages/entity-graph-core/src/adapters/flint-live.integration.test.ts` now requires built FRF artifacts and throws if they are missing, but instantiates the adapter over `createFlintLoopbackSpine()`. | Retain useful adapter compatibility coverage; add a distinct real-network, real-broker gate. A real imported class is not live infrastructure. |
| All ADR-009 materializer work remains to be implemented | The prior child phase `adr-009-replication-materializer-and-pem-publication` lists five completed changes and later executed test evidence; its nested completion dimensions contradict later top-level certification fields. | Reconcile source receipts and actual remaining gaps before rebuilding p37 work or treating the ledger as certification. |
| No TLS exists anywhere | The repository includes a historical Caddy TLS decode harness; that does not supply the production DNS/certificate/secret lifecycle. | Scope the missing work to portable production deployment and renewal/rotation proof. |

## Findings directly supported by the inspected tree

- `compose.yml` specifies hosted media without forwarding the required LiveKit
  settings or `JWT_ISSUER`; Gate's build context is a developer absolute path.
  Inline database/broker credentials, moving image tags and a root SurrealDB
  container also require profile-specific remediation.
- Iggy's compose health command ends in `|| exit 0`, so its health result can
  report success after the broker check fails. A production dependency gate must
  detect a genuinely unavailable broker.
- SSR selects sovereign media while the current media record still lacks a
  successful decode receipt. A warning/default is not proof of a hard enablement
  guard; inspect actual boot and routing behavior before calling it gated off.
- `subscribe_mux.rs` and `cdc_integration.rs` are explicitly labeled vacuous.
  The former is an outline; the latter neither produces its required insert nor
  asserts delivery. Neither can support release claims.
- Forge's LISTEN broadcast skips lagged events (`listen/watch.rs:79–88`). Its
  source has no durable catch-up protocol. Migrating it requires explicit replay,
  cursor expiry and resnapshot behavior rather than just changing the RPC name.
- Quarry's RLS filter checks that a PK re-query returns a row, then forwards the
  original event (`crates/fdb-app/src/lib.rs:107–122`). Verify payload projection,
  truncated notifications and deletion semantics separately: existence checking
  alone does not reconstruct a full payload, and a deleted row cannot pass a
  normal post-delete SELECT. These are required acceptance cases in C06.
- ADR-009 already specifies a 5,000 ms protected-delivery revocation bound and
  a single authoritative feed for clinical rows. Preserve these constraints;
  table-watch work must not become a competing clinical replication writer.

## Limits of this review

- No builds, runtime tests, registry queries, remote-main comparisons or production
  probes were performed. Build health and registry availability remain unverified.
- The p38 reflection records a dev-endpoints `E0063` failure; recheck it against
  the selected candidate before calling it a current reproduced defect.
- FRF has pre-existing edits in three shape files. Planning does not alter them;
  implementation assessment must capture the eventual source revision and diff.
- ASO/prior-auth was not found at the assumed sibling path. ADR-009 and the p37
  artifacts describe its boundary, but a current ASO checkout/deployment receipt
  is required for its gate. UAR's embedded PEM copy is not evidence that every
  UAR deployment consumes the same artifact.
- There is no new finding that a payload has actually leaked or that a production
  outage occurred. The plan calls for negative proofs at the identified boundaries.

## Evidence map

Paths below are relative to the named repository; local roots and revisions are
recorded in `entry-state.json` for reproducibility.

| ID | Repository / source | Used by |
|---|---|---|
| E01 | FRF `AGENTS.md`, `CLAUDE.md`, `.kbd-orchestrator/constraints.md` | Local-only tests, architecture, frozen v1, production feature rules |
| E02 | FRF `proto/flint/v1/entity.proto`, `crates/frf-gateway/src/entity_grpc_service.rs` | Existing single-entity contract |
| E03 | Forge `crates/fdb-realtime/src/lib.rs`, `listen/watch.rs`, `crates/fdb-gateway/src/realtime_source.rs` | Missing table watch and current fallback |
| E04 | FRF `compose.yml`, `k8s/overlays/ssr/gateway.yaml`, `crates/frf-gateway/src/config/mod.rs` | Deployment mismatches |
| E05 | FRF `crates/frf-gateway/src/authz_backend.rs`, `docs/SECURITY.md` | Authorization profile discrepancy |
| E06 | Gate `crates/flint-gate-core/src/auth/jwt_mint.rs`, FRF `crates/frf-identity-ory/src/verifier.rs` | Token and key compatibility |
| E07 | FRF `crates/frf-bridge-matrix/src/{client,lib}.rs`, `crates/frf-gateway/src/federation.rs` | Implemented bridge versus stale logs |
| E08 | FRF `crates/frf-gateway/tests/subscribe_mux.rs`, `crates/frf-postgres-cdc/tests/cdc_integration.rs` | Vacuous guards |
| E09 | FRF `.github/workflows/ci.yml`, `dagger/codegen.ts`, `docs/decisions/adr-003-ffi-codegen-versions.md` | CI policy and codegen repair |
| E10 | PEM `packages/entity-graph-core/src/adapters/flint-live.integration.test.ts`; FRF `sdks/ts/package.json` | Package and live-consumer evidence gap |
| E11 | FRF `docs/decisions/adr-009-aso-runtime-integration.md`; prior p37 child `progress.json` | Protected replication, existing work, evidence contradictions |
| E12 | FRF `docs/PHASE-36-DECODE-RESULT.md`, `docs/SECURITY.md`, `docs/RELEASE-SIGNOFF.md` | Media and release evidence needing renewal |
| E13 | FRF previous phase `reflection.md` | Known build, ledger and review limitations |
| E14 | Forge `crates/fdb-app/src/lib.rs`, `migrations/0007_change_notify.sql` | Per-event RLS and fallback semantics |
