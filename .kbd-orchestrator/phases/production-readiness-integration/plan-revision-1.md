# Production readiness integration plan

Date: 2026-09-15 · Phase: `production-readiness-integration`

**Deliverable:** a concrete remediation plan seeded by `kbd-new-phase`.
**Lifecycle:** `assessment_ready`; formal assessment, OpenSpec registration and
implementation have not run. C01–C15 below are candidate work packages, not
completed changes. Read [feedback-review.md](feedback-review.md) for corrections
and [goals.md](goals.md) for the phase outcomes.

## 1. Release strategy and non-negotiable constraints

Establish hosted production readiness first, then certify protected replication,
federation and sovereign media against their own gates. All remain in this plan;
an earlier hosted release cannot certify the remaining capabilities.

- All tests run locally against a locally composed stack. No workflow dispatch,
  push-to-test, CI rerun, or Dagger CI test stage can satisfy a gate. A local
  Linux VM/container topology is acceptable; a hosted CI runner is not.
- Build and package checks cover the exact candidate source and feature sets.
  Historical green builds, mock tests and percentages do not establish readiness.
- Keep `proto/flint/v1/*.proto` frozen. The proposed new table-watch service goes
  in a new versioned contract, with compatibility checks for existing v1 clients.
- Keep domain and application dependencies inward and each adapter on one port.
  Use existing pinned dependencies and ADRs; recheck current official docs with
  Context7 before implementation-specific library/API decisions.
- Production artifacts exclude `dev-endpoints`/`DEV_NO_AUTH`. A restricted
  deployment's `verified-identity` mode must not be promoted to a general-purpose
  object authorization mechanism.
- Preserve ADR-009's maximum 5,000 ms protected-delivery revocation bound, one
  authoritative feed per clinical entity, and local-only data egress exclusions.
- No production deployment, public package publication or implementation occurs
  during this planning invocation. Each later phase boundary follows CLAUDE.md.

## 2. Owners and integration boundaries

Owners identify repository responsibilities, not agents or assignments sent to people.

| Owner | Responsibility | Consumer acceptance |
|---|---|---|
| Fabric | Versioned watch API, CDC/spine delivery, gateway authorization, SDK artifacts, deployment profiles | Forge and PEM consume actual candidate artifacts over real transports |
| Gate | Real session validation, audience-bound signing/JWKS, membership grants, revocation and proxy boundaries | Fabric/Forge accept intended grants and reject stale, wrong-scope or wrong-audience grants |
| Forge | Generated watch client, subscription factory, row visibility and delivered payload projection | GraphQL subscription observes visible committed changes and hides unauthorized data |
| PEM | SDK adapter integration, normalized graph updates, disconnect/replay/epoch handling | Real service events update graph and lists without old-session writes or duplicates |
| ASO/prior-auth | Approved replica schema, application membership, materializer and browser/native runtime | Protected rows/checkpoints/PEM revisions remain coherent through revoke, switch and crash |
| Deployment owner | Runtime topology, DNS/TLS, secrets, images, backup and recovery | The declared profile is reproducibly installed and meets its own readiness checks |
| Other consumers, including UAR | Inventory actual FRF package and protocol dependencies in C01 | A consumer is claimed compatible only after an identified version passes its contract lane |

### Data paths to prove

1. **Forge changes:** Postgres commit → FRF CDC → durable broker → type watch →
   Forge per-event RLS/projection → authenticated GraphQL socket.
2. **PEM realtime:** authenticated SDK publish / accepted entity change → real
   FRF transport and broker → generated SDK/adapter → normalized graph and lists.
3. **ASO relational replica:** Kratos session → Gate membership/grant → FRF shape
   facade → approved Electric projection → atomic local SQL/checkpoint commit →
   coherent PEM publication. Do not route the same clinical rows through path 1/2.
4. **Media/federation:** independently authorized media and external-event lanes,
   with profile-specific recovery and revocation proof.

## 3. Ordered work packages

### C01 — Establish source, capability and build baselines (P0)

**Owner:** Fabric; Gate/Forge/PEM supply their candidate revisions. **Depends:** none.

- Inventory each repository's commit, dirty diff, lockfiles, generated contracts,
  image/package versions, effective feature/env profiles and actual consumers.
  Locate the ASO checkout; reconcile p37 receipts and contradictory ledger fields.
- Build/check Fabric, Gate and Forge locally; reproduce the reported dev-endpoints
  error and fix only demonstrated failures. Include default, dev-endpoints and
  deployed shape-facade/authorization feature combinations.
- Repair Dagger's obsolete Dart generation path to follow ADR-003. Remove test
  execution from all CI-reachable workflows, including decode and Playwright
  stages; preserve their required scenarios as local entry points.
- Classify every shipped capability as implemented, runtime-proven, unsupported
  or blocked. Record build failures separately from missing runtime evidence.

**Exit:** reproducible local build/lint/typecheck/package checks on a source
manifest; no CI test runner remains reachable; an agreed capability matrix and
test workload exist. Existing v1 generation remains reproducible.

### C02 — Portable, truthful deployment profiles (P0)

**Owner:** Fabric deployment; Gate supplies a versioned image contract. **Depends:** C01.

- Separate a local integration profile from production profiles; replace absolute
  sibling build paths with pinned images or documented configurable local paths.
- Supply and validate issuer, audience, JWKS route, LiveKit settings, database and
  broker credentials. Use generated disposable secrets for local fixtures and a
  declared secret source in production; never commit actual credentials.
- Pin dependency images; remove root/default-credential production settings and
  unnecessary host ports. Correct the Iggy health command that masks failures.
- Provide a hosted production profile. Keep sovereign and federation explicitly
  experimental until their gates pass. Reject incomplete selected configurations.
- Make dependency readiness semantic: wrong/missing keys, unavailable broker or
  required authorization service must prevent a false-ready result.

**Exit:** clean-clone boot of the release artifact with declared inputs; authenticated
readiness passes. Missing required inputs and stopped dependencies fail visibly.
Render each supported production profile and verify enabled lanes match its claims.

### C03 — Identity and authorization across deployed lanes (P0)

**Owner:** Gate + Fabric; Forge/ASO verify their boundaries. **Depends:** C01, C02.

- Trace real login/session → Gate grant → FRF verification using the deployed
  key algorithm, `kid`, issuer, audience, tenant, subject and expiry contracts.
  Exercise key rotation, unknown keys, expiry and authority failure.
- Audit `verified-identity` versus Keto on spine, entity, agent, shape and media
  egress. For generic object-private traffic, require an actual authorization
  authority or reject that traffic in the profile; no implicit reliance on RLS
  where a raw stream never re-queries Postgres.
- Prove same-tenant/different-subject and cross-tenant denial, refreshed token
  behavior, logout, membership removal, active-stream closure and stale cache
  refill races. Preserve Gate's ownership of fresh ASO authority and FRF cleanup.
- Enforce network paths so consumers cannot bypass the authority proxy and reach
  FRF/Electric/operator endpoints directly where the protected profile forbids it.

**Exit:** real Gate-issued credentials work without dev bypass; required denials
pass on every exposed lane. Protected ASO delivery ends within 5,000 ms of
authoritative revocation/expiry through the last protected byte, including queues.

### C04 — Freeze the table-watch contract with Forge (P0)

**Owner:** Fabric contract; Forge co-owns consumer semantics. **Depends:** C01, C03.

- Specify `WatchEntityType` in a new versioned proto service (proposed v2).
  Preserve single-ID v1 `WatchEntity`; do not edit frozen v1 to add the method.
- Define server-derived tenant scope, schema/table mapping, authenticated caller,
  entity keys including composites, operation, current/previous projections,
  event identity, commit position, resume cursor and bounded subscription lifetime.
- Decide start position, ordering scope, at-least-once delivery/deduplication,
  acknowledgement/checkpoint ownership, retention and expired-cursor errors.
  Specify snapshot-to-stream handoff so a reconnect cannot silently miss writes.
- Specify resource limits, cancellation, lag behavior, unavailable/denied errors,
  revocation and safe deletion/tombstone semantics. Arbitrary filter pushdown is
  outside the first contract unless a consumer requires and verifies it.
- Record the version/feature negotiation and supported old/new client matrix.
  Generate and freeze the contract before dependent SDK/client implementation.

**Exit:** reviewable proto/ADR/OpenSpec contract and producer/consumer scenarios;
both repositories agree on semantics, v1 compatibility checks pass, and no client
depends on a handwritten copy or an assumed rename.

### C05 — Implement durable Fabric type watches (P0)

**Owner:** Fabric domain/ports/app, CDC/broker adapters and gateway. **Depends:** C04.

- Follow the real CDC ingestion path to the broker and type-filtered subscription;
  do not back the new endpoint with only the in-memory entity store/broadcast.
- Add the application use case and port seam; compose one-port adapters at the
  gateway. Verify event mapping, transaction boundaries and tenant/type routing.
- Apply subscribe-time and required per-event authorization; close on revoke or
  authority failure. Bound buffers and subscriber lifetime; release resources on
  disconnect and cancellation, including during an awaited authorization check.
- Persist/derive resumable positions; report lag and retention exhaustion
  explicitly. Define consumer deduplication and recovery rather than promising
  global exactly-once delivery.

**Exit:** real INSERT/UPDATE/DELETE transactions reach two type subscribers;
another table/tenant receives nothing. Gateway/broker reconnect, cancellation,
slow-consumer overflow and expired-cursor scenarios satisfy the frozen contract.

### C06 — Replace Forge's Fabric stub and verify safe payloads (P0)

**Owner:** Forge realtime/app/gateway. **Depends:** C03, C05.

- Bind the generated client, map stream errors and versions, and implement the
  Fabric source under its existing source-selection seam.
- Keep the authoritative per-event RLS re-query and authorized column projection.
  Verify that delivered records use permitted current data rather than merely
  forwarding a raw event after an existence check.
- Cover denied rows, column restrictions, changed primary keys, composite keys,
  truncated notifications and deletes. Choose a safe tombstone/invalidation
  contract: a deleted row cannot pass an ordinary post-delete SELECT, and raw
  `old_record` is not an authorization substitute.
- Exercise listen/fabric parity for their common guarantees and document the
  durability difference. Preserve `listen` as the default until the full Fabric
  lane passes; support explicit rollout and rollback without duplicate writers.

**Exit:** GraphQL clients observe permitted database changes through Fabric,
including defined delete behavior, with no silent empty-stream fallback. Wrong
identity/tenant and permission changes deny correctly. Close OQ-FRF-1 only after
the consumer proof passes, not merely when the proto compiles.

### C07 — Publishable SDK artifacts and actual PEM network integration (P0)

**Owner:** Fabric SDKs + PEM. **Depends:** C03, C04, C05.

- Build/package the TS SDK and entity-management adapter from frozen contracts;
  verify every advertised export, declaration and runtime file exists in the
  tarball, including ESM/CJS claims. Check Rust and generated/native support matrix.
- Install the candidate package in an isolated consumer, with no sibling-source
  import shortcuts. Use a real authenticated FRF client and broker in PEM's
  network suite; preserve loopback tests under an accurate label.
- Prove mutation/event-to-graph/list updates, correlation and duplicate handling,
  reconnect/cursor recovery, logout/epoch fencing and tenant switches.
- Include the previously unproven Rust live-publish example and consumers found
  in C01. Bind package checksums and FRF/PEM revisions into evidence receipts.

**Exit:** packaged consumers deliver observable graph state over real transport;
the required network lane fails if artifacts or services are missing. A removed
delivery route must make the proof fail. Public registry availability remains a
separate publication result, not inferred from a successful local package build.

### C08 — Complete ADR-009 protected replica integration (P0 for ASO)

**Owner:** ASO + PEM; Fabric/Gate supply authority and shape boundaries.
**Depends:** C03, C07; starts with source/evidence reconciliation from C01.

- Reuse p37 runtime-scoped state, disposal barriers and checkpoint work where
  present. Resolve ledger contradictions from artifacts and current source;
  neither stale BLOCKED fields nor unqualified CERTIFIED labels settle the gate.
- Prove persisted row transitions across approved shapes, authorized continuation,
  interrupted bodies, handle expiry/refetch and upstream failure.
- Crash between row application, checkpoint commit and graph publication. On
  restart, require atomic SQL/checkpoint state and a coherent PEM revision.
- Prove logout/reload and identity/practice switches fence in-flight writes and
  remove the old graph; no credentials in Zustand and no local-only/embedding
  egress. Exercise browser and each claimed native replica implementation.

**Exit:** deployment-specific Kratos → Gate → FRF → Electric → materializer → PEM
receipt with bounded revocation and crash consistency. No competing watch/CRDT
writer owns those clinical rows. Missing ASO source/topology keeps this gate open.

### C09 — Meaningful local integration gates (P0)

**Owner:** Fabric verification; consumer repositories supply scenarios.
**Depends:** C02, C03; extend as C05–C08 land.

- Replace the vacuous mux test with an authenticated real publish/receive assertion.
  Replace the CDC outline with actual transactions and asserted broker/client
  delivery. Do not count zero events, sleep completion or a printed count as proof.
- Provide deterministic isolated fixtures, unique tenant/channel/slot identifiers,
  cleanup limited to those fixtures, deadlines and explicit prerequisite failures.
- Required release suites must fail on missing services or skipped scenarios.
  Preserve optional developer suites without confusing them with release evidence.
- Demonstrate selected guards fail when delivery, authorization or checkpoint
  behavior is deliberately broken in an isolated test candidate, then pass restored.

**Exit:** reproducible local suite with expected scenario count, zero required
skips and captured positive/negative evidence; no CI result is used for closure.

### C10 — Operations, recovery and deployment lifecycle (P0)

**Owner:** Fabric deployment + dependency owners. **Depends:** C02, C03, C05, C09.

- Supply reproducible ingress/TLS/DNS, pull-secret and secret-manager integration
  using the deployment's existing platform. Specify any externally managed
  prerequisite and validate it; do not add a second cluster-management system.
- Exercise key/certificate/credential rotation, resource limits, backpressure,
  readiness, shutdown draining and rolling replacement without silent event loss.
- Measure queue depth/lag, CDC slot/WAL growth, delivery delay, auth denial/errors,
  reconnects and cancellation cleanup without exposing sensitive payloads.
- Prove broker/database/checkpoint backup and restore; include schema migration
  rollback and version compatibility. Alert on no-progress and retention risk.

**Exit:** local recovery/restore and rotation receipts; operations runbook; explicit
measured latency, capacity, retention, RTO and RPO against predeclared targets.

### C11 — Hosted media production proof (P0 for hosted media)

**Owner:** Fabric LiveKit integration + deployment. **Depends:** C02, C03, C09.

- Boot the hosted profile with the intended LiveKit server configuration and real
  room grants. Prove two browser participants exchange decoded audio/video,
  reconnect and leave, and unauthorized participants cannot join/receive.
- If cross-node relay is advertised, enable its required feature and prove two
  gateway instances exchange the intended signals. Otherwise mark that capability
  unavailable in the profile instead of assuming one-node proof covers it.

**Exit:** observable hosted media receipt with positive decoding, room/tenant
isolation and the authorization lifetime required by the workload.

### C12 — Federation functionality and recovery (P1, separate release gate)

**Owner:** Fabric Matrix/ATProto adapters and deployment. **Depends:** C03, C09, C10.

- Reconcile gateway messages and SECURITY/API documentation with actual adapters.
  Exercise existing Matrix inbound/outbound and configured ATProto inbound/PDS
  outbound against local services or faithful protocol servers, labeling the latter.
- Prove reconnect, durable offset/cursor recovery, duplicate suppression, echo-loop
  prevention, stable tenant/channel mapping and downstream authorization.
- Add or repair only behavior shown missing. Capture actual target-server-version
  proof before certifying that deployment; protocol fixtures alone are insufficient.

**Exit:** supported directions work after restart without cross-tenant delivery or
unbounded duplication. Unconfigured directions are visibly unavailable.

### C13 — Sovereign media decode and protected fan-out (P1, separate release gate)

**Owner:** Fabric str0m/signaling/room lifecycle. **Depends:** C03, C09, C10.

- Port the existing proof entry point to a locally controlled Linux topology with
  routable ICE/TURN candidates. Retain logs and stats before teardown; no CI run.
- Reproduce the latest receiver/room/keyframe failure with current source. Trace
  membership, negotiated tracks/MIDs, PLI/FIR, keyframes and actual media routing;
  do not assume the historical diagnosis still explains every failure.
- Require room membership and tenant separation on media egress. Exercise two
  peers, late join, reconnect and multiple rooms, with receiver-side decode evidence.
- For protected workloads, enforce bounded expiring/revocable room grants and
  remove revoked peers from fan-out within the required bound.

**Exit:** repeated local receiver `framesDecoded > 0`, ongoing frame/audio progress,
correct-room-only delivery, bounded revocation and no task/socket leak. Enable a
production sovereign profile only after this receipt and its security review pass.

### C14 — Honest admin and platform support (P1)

**Owner:** Fabric admin/FFI SDKs; Gate identity integration. **Depends:** C03, C07.

- Evaluate standalone admin login against ADR-004 and the actual Gate/IdP surface.
  Adopt an accepted login design before implementation; ASO's Kratos/Gate route
  does not inherit an automatic Hydra requirement from the old standalone proposal.
- Prove login, expiry, logout and access controls for any interactive-admin claim.
  A restricted operator JWT tool can have its own limited profile, not an OIDC claim.
- Recheck Dart async-generation limitations against the pinned/superseding ADR;
  fix and prove transport before claiming parity. Preserve working CRDT bindings.
  Test each advertised SDK/platform, including actual native transport where claimed.

**Exit:** explicit per-platform support matrix. Interactive admin and Dart async
remain open capability gates until delivered; documentation alone does not close them.

### C15 — Current security audit and release evidence (P0, per profile)

**Owner:** Fabric release; Gate/Forge/PEM/ASO attest their tested seams.
**Depends:** applicable gates above; hosted can be evaluated before C12/C13 finish.

- Re-audit current source and effective deployment configuration, including
  authorization modes, stream lifetime, privacy projection, logs, media and SDKs.
  Record reviewer identity/method and unavailable independent-review coverage.
- Resolve all critical/high findings for the selected production profile; do not
  borrow the phase-17 signoff. Keep historical evidence and write a dated successor.
- Bind every claim to source/diff digest, package/image digest, lockfiles, topology,
  configuration fingerprint, test command, timestamp, exit status and artifact hash.
- Reconcile RELEASE-SIGNOFF, SECURITY, API, ENVIRONMENT, CHANGELOG and downstream
  integration docs. Document rollout, fallback and rollback compatibility.

**Exit:** explicit GO/NO-GO per profile with all required local receipts. Publication
status is separate and only changes after the actual registry/deployment result.

## 4. Dependency order and release gates

```text
C01 → C02 → C03 → C04 → C05 → C06 (Forge)
                      └───────→ C07 (SDK/PEM) → C08 (ASO)
             C02+C03 → C09 → C10
                       ├────→ C11 (hosted media)
                       └─ C10 → C12 (federation), C13 (sovereign)
                 C03+C07 → C14 (admin/platform support)
             Applicable completed proofs → C15 (profile signoff)
```

| Gate | Required proof | Does not certify |
|---|---|---|
| Hosted realtime + Forge + PEM | C01–C07, C09–C10, applicable C14, C15 | ASO protected replication, federation, sovereign media, unsupported SDK transports |
| Hosted media | Hosted gate + C11 | Sovereign decode or untested cross-node relay |
| ASO protected replication | Hosted data prerequisites + C08 + C15 for exact topology | Protected agent/media lanes without their own egress proof |
| Federation | Applicable core gates + C12 + C15 | Unconfigured directions or different external server versions |
| Sovereign media | Applicable core gates + C13 + C15 | Full ASO media safety unless its revocation cases pass |
| Interactive admin / Dart async | C14's functional proof + C15 | Features listed only as deferred |

The phase is not fully closed while agreed goals remain unmet. A formally accepted
scope reduction may permit a narrower release, but never turns an unsupported
capability into a completed implementation or a passed certification gate.

## 5. Validation targets and evidence rules

- **Security:** zero unauthorized events/rows/media in negative cases; protected
  ASO revocation maximum 5,000 ms, not a percentile or average. Check the last
  buffered byte delivered, not just token denial on a new connection.
- **Durability:** every acknowledged committed event within configured retention
  is recoverable under the contract; duplicates are tolerated only with explicit
  deduplication. An expired cursor produces a visible resnapshot requirement.
- **Proposed capacity baseline:** 1,000 concurrent subscriptions, 100 committed
  changes/second for 60 minutes, delivery p95 ≤1 s and p99 ≤3 s on a documented
  local resource allocation. Freeze or revise this target in C01 before running;
  it is a planning target, not an existing product guarantee.
- **Recovery:** choose retention/RPO/RTO from deployment requirements in C01;
  refuse C10/C15 closure while those targets are unspecified or unmeasured.
- **Required proofs:** no silent skips, no loopback client replacing the production
  path, and no synthetic JWT substituted for Gate in the identity-chain gate.
  Synthetic data/isolated fixture credentials are appropriate for local proofs.
- **Media:** sample decode counters over time, retain nonempty gateway and browser
  evidence, and exercise late join/reconnect/isolation; ICE alone is insufficient.
- **Freshness:** changes to source, generated contract, dependencies, package,
  identity configuration or topology invalidate affected receipts. Re-run affected
  integration gates before release; keep unrelated evidence with explicit scope.

## 6. Handoff and first action

Next: `/kbd-assess production-readiness-integration`.

Use this plan to assess C01 first, reconcile the prior p37/p38 work and register
bounded OpenSpec changes with unique IDs and scenario-level acceptance criteria.
Split oversized work packages during registration without losing cross-repo gates.
Keep legacy KBD authority for this phase; migration is a separate decision.

Phase initialization uses the skill's step-by-step path because the installed
helper preserves the old `path[]` and completion flags. The new top-level path is
explicitly `[production-readiness-integration]`, with prior state retained in
`entry-state.json`; the new phase is not nested under the old media phase.
