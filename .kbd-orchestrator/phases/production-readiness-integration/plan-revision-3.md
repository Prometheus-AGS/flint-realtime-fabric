# PLAN: production-readiness-integration

Project: flint-realtime-fabric · Date: 2026-09-16 · Revision: 3
OpenSpec available: YES · Changes proposed: 23 · Implemented: 10

## Outcome and authority

Deliver the shortest continuously usable prototype path from a local PGlite
mutation through Forge and PostgreSQL, then back through ElectricSQL/Fabric into
PGlite and PEM. Keep the database-triggered Forge GraphQL subscription path
working in the same profile. After that complete user flow exists, add durable
offline replay and only then resume ASO, media, federation, platform and release
qualification. The phase retains 23 changes, but revision 3 redefines c012 and
c013 around the runnable prototype instead of package breadth and unrelated CRDT
storage.

ElectricSQL is the downstream PostgreSQL-to-client replication path. PGlite
writes do not travel upstream through ElectricSQL: the prototype records the
optimistic mutation in PGlite, submits it through Forge's authenticated mutation
API, and reconciles from the canonical Electric shape delivered through Fabric's
authorized shape facade. The shorthand `PGlite -> ElectricSQL -> database` is
therefore implemented as the complete bidirectional product loop rather than a
nonexistent Electric upstream-write API.

The operator requested planning. No production implementation, deployment,
publication or KBD runtime migration is authorized by this artifact. OpenSpec
structures are proposals; dependent contracts are not frozen by writing them.
The existing legacy ledger remains 0/0 until its authority decision is resolved;
the 23 proposal records are indexed in planned-changes.json and OpenSpec, not
misrepresented as completed or runtime-registered implementations.

Inputs: [assessment](assessment.md), [bounded analysis](analysis.md),
[library candidates](library-candidates.json), [decision log](decision-log.md).
The prior plan is preserved as plan-revision-1.md. Baseline source and external
excerpts remain under evidence/. No earlier certification is inherited.

## Decisions that remain open

Q1–Q7 were asked during Analyze and remain unanswered. Planning does not accept
the suggested defaults. c003 is a prerequisite for freezing their dependent
requirements; c001 and c002 can proceed independently when execution is requested.

| Question | Decision required | Dependent work |
|---|---|---|
| Q1 | First-release profiles versus whole-phase scope | c003 release matrix; c017–c023 release inclusion |
| Q2 | Deployment topology, existing infrastructure and application consumers | c004–c005, c013, c015–c021 |
| Q3 | Required browser/server/native clients and module formats | c012 package matrix; c022 platform proof |
| Q4 | Durable change replay versus current-state recovery; history duration | c006–c012 and operational recovery |
| Q5 | Tenant/database mapping, schema/key types, deletes and audit consumers | c005–c012 |
| Q6 | Subscription/rate/latency/retention/RTO/RPO targets | c003 contract; c007–c010, c013, c016 |
| Q7 | ASO memory/revocation requirements and browser/native priority | c014–c015 |
| Q8 | Legacy KBD authority versus project-wide migration | Stage recording only; separate from engineering work |

Q8 does not block executing independently authorized source work. Do not invoke
typed KBD mutation that implicitly migrates history. Track local proposal/task
evidence without fabricating canonical completion flags. No new migration change
is bundled into this production-readiness phase.

c003 also closes Analyze's explicit research backlog: pinned identity/policy,
storage, deployment, media, federation and native SDK API/maintenance fit.
Candidates are provisional; pg_walstream requires unresolved maintenance/security
verification. Alternative CDC services were not evaluated. If a selected design
fails qualification, revise the affected proposal before implementing it.

## Fixed constraints and acceptance boundaries

- Keep frozen proto/flint/v1 byte-identical. New RPCs use a newly accepted
  versioned contract; generated clients follow it. Fabric proto v2 does not
  imply Connect-ES v2. Preserve ADR-001/003/009 unless explicitly superseded.
- Domain/application dependencies remain inward; each adapter implements one
  port. Compose adapters only in frf-gateway. New read/populate/checkpoint seams
  need separate one-port adapters. Record public type/port semver changes.
- Complete user-facing functionality before adding isolated tests. The first
  behavioral test after c011-c013 implementation is one local full-stack flow:
  PGlite mutation -> Forge API -> PostgreSQL commit -> Electric/Fabric delivery
  -> PGlite reconciliation -> PEM update. CI only builds, lints, typechecks,
  formats and packages. Unit or component tests cannot close these slices.
- Generic object streams require current object authorization; identity alone
  does not grant object access. Forge delivers the authorized row projection,
  not a raw event merely allowed by an existence check.
- ASO keeps one authoritative clinical shape feed. Generic watch/CRDT delivery
  must not become a competing writer for those same clinical rows.
- Preserve ASO's incremental RSS ceiling 536870912 bytes and heap ceiling
  268435456 bytes unless the operator explicitly changes its accepted contract.
  Existing measured RSS failure keeps production adoption experimental.
- The ASO maximum 5000 ms ends at the last server-produced protected frame or
  cancellation preventing the next frame, followed by subsequent-request denial.
  Start is authoritative ASO membership/session-denial commit or verified expiry.
  Direct Kratos revocation starts at documented observation/ASO-denial commit.
  Kernel/proxy buffers, network transit and client receipt are excluded.
- No global exactly-once promise. c006 defines ordering, stable identity,
  cursor inclusivity, source epoch, transaction durability, resync and deduplication.
  Existing CDC counters and broker polling auto-commit do not meet that proof.
- Files remain at most 500 lines. Preserve the three pre-existing shape edits
  identified in entry-state.json; reconcile ownership before touching them.
- Use synthetic records and run-owned fixture secrets. No sensitive data enters
  logs or receipts. Do not affect unrelated services/volumes in sibling projects.

## Ordered changes

Complexity is a planning estimate per bounded session, not a delivery promise.
All model classes remain frontier because project.json has no model_policy;
the complexity classifier is recorded separately. Agent labels recommend future
execution only: no agents, tasks or messages were dispatched by this plan.
Any L slice that cannot fit a bounded session must be split with dependencies
and retained exit criteria before work continues; diagnosis alone cannot close it.

### 1. pri-c001-build-policy — Restore the declared build matrix and local-only testing policy

- **Owner / scope:** Fabric; C01. **Priority:** P0.
- **Depends on:** none. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** M; **Complexity score:** Medium; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Fix the demonstrated missing SignalEnvelope subject and reconcile MSRV. Remove CI-reachable test execution while preserving local entry points; correct stale Dart-generator guidance to ADR-003.
- **Acceptance A:** Default, dev-endpoints and shape-facade checks plus applicable Clippy, format and SDK typecheck succeed locally against recorded source hashes.
- **Acceptance B:** Inventory every workflow/Dagger call path: CI performs build/lint/typecheck/format/package only; each removed runtime gate has a documented local invocation and is exercised by its owning later change.

### 2. pri-c002-local-fixtures — Provide an isolated local integration runner

- **Owner / scope:** Fabric integration; C09. **Priority:** P0.
- **Depends on:** pri-c001. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** M; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Create a namespaced composed fixture and mandatory-prerequisite runner with disposable credentials, deadlines, receipts and ownership-safe cleanup. Replace the mux outline with real publish/receive; reserve CDC semantics for c008.
- **Acceptance A:** A real authenticated publish reaches a subscribed gateway client; disabling delivery makes that assertion fail before restoration passes.
- **Acceptance B:** Missing prerequisites, zero required scenarios and skipped required tests exit nonzero; cleanup touches only the run-owned containers/volumes/slots.

### 3. pri-c003-scope-research-contract — Resolve release decisions and complete dependency qualification research

- **Owner / scope:** Fabric coordinator + Gate/Forge/PEM/ASO owners; C01 + Analyze backlog. **Priority:** P0.
- **Depends on:** none. **Decision inputs:** none to start; Q1–Q7 are closure outputs.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Resolve Q1–Q7 with provenance and research the named identity/policy, persistence/deployment, media/federation and platform backlog in bounded passes. Inventory source/package/image revisions and runtime profiles. Record Q8 separately; KBD migration is not part of this change. Research and decision facilitation may start immediately; unresolved Q1–Q7 block closure and dependent implementation, not this change starting.
- **Acceptance A:** An accepted release matrix names required profiles, clients, source schemas/keys/tenants, historical/delete semantics, scale/retention/RTO/RPO, ASO budgets and profile-specific authorization lifetime endpoints.
- **Acceptance B:** Pinned API, compatibility and maintenance/security evidence exists for each retained dependency; unsupported choices trigger a revised proposal, not invented feasibility. Reconcile Assess provenance finding through focused independent review.

### 4. pri-c004-deployment-profiles — Make selected deployment profiles portable and truthful

- **Owner / scope:** Fabric deployment + Gate; C02. **Priority:** P0.
- **Depends on:** pri-c001, pri-c002, pri-c003. **Decision inputs:** Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** M; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Render supported full and restricted-shape profiles from pinned artifacts and explicit identity/media/secret inputs. Remove absolute developer paths and false-success health checks; reject endpoints outside profile authority.
- **Acceptance A:** Clean local installation of each selected profile reaches semantic readiness with real dependency handshakes; missing issuer/keys/broker/auth authority fails visibly.
- **Acceptance B:** Pinned images, declared secret sources, exposed ports and feature flags match the profile; release artifacts contain no dev authorization bypass. Each profile declares TLS termination, certificate/key/trust sources and external/internal network boundaries; valid HTTPS succeeds, invalid or missing TLS inputs fail startup/readiness, and prohibited plaintext or backend-bypass access is denied.

### 5. pri-c005-authority-lifetime — Enforce identity and authorization on selected exposed lanes

- **Owner / scope:** Gate + Fabric; Forge/ASO boundary owners; C03. **Priority:** P0.
- **Depends on:** pri-c002, pri-c003, pri-c004. **Decision inputs:** Q2, Q5, Q7 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Verify real Gate credentials and current object authorization on generic streams, and preserve Gate/ASO fresh authority on restricted shape streams. Check expiry, rotation, revoked grants, cancellation and bypass paths.
- **Acceptance A:** Real Gate-issued tokens allow only intended issuer/audience/tenant/subject and object; same-tenant unauthorized subjects, cross-tenant access, authority loss, key rotation and stale refill races are exercised locally.
- **Acceptance B:** ASO authoritative commit/expiry to last server-produced protected frame or cancellation preventing the next frame is at most 5000 ms, followed by new-request denial; direct Kratos revocation uses the documented observation/ASO-denial start. All other lanes meet the lifetime contract accepted in c003.

### 6. pri-c006-watch-contract — Freeze the versioned watch and recovery contract

- **Owner / scope:** Fabric contract + Forge/PEM consumers; C04. **Priority:** P0.
- **Depends on:** pri-c003. **Decision inputs:** Q4, Q5, Q6 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Specify a new versioned WatchEntityType contract, canonical typed payload/key mapping and compatibility with immutable v1. Separate source LSN/epoch, stable event identity, partition offset and client checkpoint; specify snapshot barrier, ordering, duplicates, retention, lag, cancellation and authorized deletes.
- **Library:** library: cand-001; library: cand-002; library: cand-004. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Contract scenarios cover accepted schema/key/tenant shapes, transaction commit, snapshot-to-live races, restart/replay, old cursors, unauthorized history and deletion/projection; consumer acceptance is recorded before codegen.
- **Acceptance B:** Frozen proto/flint/v1 files remain byte-identical; new-version generation and old/new compatibility validation succeed. Semver impact on frf-domain/frf-ports and one-port adapter boundaries are documented.

### 7. pri-c007-broker-replay — Expose durable broker positions and commit-controlled replay

- **Owner / scope:** Fabric broker adapter; C05. **Priority:** P0.
- **Depends on:** pri-c002, pri-c006. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Replace producer-local replay counters with the accepted broker position mapping. Configure explicit commit policy instead of polling-time defaults and prove exact pinned-fork seek/replay/ack semantics, stable producer identity and cancellation.
- **Library:** library: cand-001. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Crash after poll but before durable consumer checkpoint replays the event; restart after acknowledgement respects defined cursor inclusivity and duplicate rules.
- **Acceptance B:** Producer restart does not reuse a replay position; independent consumers, partition boundaries, retention expiry and cancellation behave as the frozen contract requires. Record actual server durability/flush configuration.

### 8. pri-c008-cdc-commit-mapping — Deliver committed typed CDC changes without checkpoint gaps

- **Owner / scope:** Fabric CDC; C05. **Priority:** P0.
- **Depends on:** pri-c002, pri-c006, pri-c007. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reuse the pinned WAL parser; implement validated catalog/replica-identity mapping, schema-qualified routing and supported typed values. Couple acknowledged WAL progress to durable committed publication, including decode/transaction failures.
- **Library:** library: cand-001; library: cand-002. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Real INSERT/UPDATE/DELETE and rollback/multi-row transactions exercise enrolled non-first/composite/non-UUID keys and accepted tenant modes; unsupported mappings fail enrollment, not silent delivery.
- **Acceptance B:** Crash between publication and LSN advancement permits defined deduplication without loss; poison rows, schema change, unchanged TOAST and missing old-key data cannot be skipped beneath an acknowledged checkpoint.

### 9. pri-c009-entity-projection — Populate durable v1 entity reads and watches

- **Owner / scope:** Fabric entity projection; C05 / F01. **Priority:** P0.
- **Depends on:** pri-c004, pri-c005, pri-c008. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Compose a durable entity projection and population/checkpoint seam following one-port-per-adapter. Establish snapshot-to-WAL catch-up and readiness so existing GetEntity/WatchEntity reflect production ingestion.
- **Library:** library: cand-001; library: cand-002. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** A database commit reaches existing v1 GetEntity and WatchEntity through real adapters; snapshot overlap does not omit or duplicate state incorrectly.
- **Acceptance B:** Gateway/projector restart preserves state and cursor coherence; delete removes current state; v1 wire remains unchanged and required per-event object authorization is applied.

### 10. pri-c010-type-watch — Serve authorized resumable WatchEntityType streams

- **Owner / scope:** Fabric application + gateway; C05. **Priority:** P0.
- **Depends on:** pri-c005, pri-c006, pri-c007, pri-c008, pri-c009. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Implement the frozen type-watch use case and transport using durable broker delivery and projection semantics. Enforce subscribe-time/per-event authority, bounded buffers, explicit resync and teardown. Generate/export and compile-check the tonic Rust client in frf-proto for c011 here; c012 owns separate non-Rust package generation.
- **Library:** library: cand-001; library: cand-002. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Two authorized subscribers receive committed changes of the enrolled type; other schemas/types/tenants and unauthorized same-tenant subjects receive no protected payload.
- **Acceptance B:** Snapshot/subscribe races, reconnect, expired cursor, slow-client lag, cancellation during awaited authorization and revocation meet the contract; resources return to baseline.

### 11. pri-c011-forge-watch — Replace Forge's Fabric stub with safe GraphQL delivery

- **Owner / scope:** Forge realtime/app/gateway; C06. **Priority:** P0.
- **Depends on:** pri-c005, pri-c010. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Finish the runnable SQL-write -> Fabric WatchEntityType -> authenticated Forge GraphQL-subscription path using the c010 tonic client and canonical payload mapping. Deliver the authorized RLS re-query projection, safe deletion/invalidation behavior and truncation reconstruction. Retain LISTEN as a selectable rollback source while Fabric becomes usable in the prototype profile.
- **Library:** library: cand-004. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** A real SQL write reaches an authenticated GraphQL subscription with only permitted fields; denied rows, composite/key changes, deletes and truncated events behave as specified.
- **Acceptance B:** Disconnect/reconnect, revocation, backend failure and switch/rollback are explicit and do not silently lose updates or double-apply them. OQ-FRF-1 closes only after this receipt passes.

### 12. pri-c012-sdk-pem-network — Deliver the runnable PGlite–Forge–Electric prototype loop

- **Owner / scope:** Forge + Fabric shape facade + PGlite/PEM adapter; C07. **Priority:** P0.
- **Depends on:** pri-c004, pri-c005, pri-c011. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Provide a runnable browser/Node prototype in which PGlite owns optimistic local state, authenticated Forge mutations write PostgreSQL, Electric consumes the committed row, Fabric's authorized shape facade protects the Electric stream, PGlite electric sync applies the canonical row, and PEM observes the resulting local-table change. Package expansion is deferred until this path works from a checked-in launch command.
- **Library:** library: cand-003; library: cand-004. Reuse the existing PGlite/PEM transport seams and the pinned SDK pipeline without requiring broad platform generation first.
- **Acceptance A:** From the runnable prototype, one create/update/delete initiated against PGlite reaches PostgreSQL through Forge and returns through Electric/Fabric to the same PGlite table and PEM entity/list without manual refresh or direct database access.
- **Acceptance B:** A second authenticated client observes the canonical database change, tenant/field projection is enforced, and switching the realtime source back to LISTEN keeps the Forge GraphQL lane usable while shape sync remains explicitly gated.

### 13. pri-c013-sync-persistence — Make PGlite outbound sync durable and restart-safe

- **Owner / scope:** PGlite/PEM sync transport + Forge mutation API; C10 / F10. **Priority:** P0.
- **Depends on:** pri-c012. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Add a durable PGlite outbox, stable mutation identity, retry/replay and canonical reconciliation around the c012 loop. Preserve optimistic rows across browser/process restart, submit through Forge when connectivity returns, and retire outbox entries only after canonical database state is observed. Fabric CRDT snapshot/op-store qualification moves to a later phase because it does not make this prototype loop usable.
- **Acceptance A:** A mutation created while Forge/Electric is unavailable survives restart, is delivered once connectivity returns, and converges through the canonical Electric/Fabric row in PGlite and PEM.
- **Acceptance B:** Retries do not create duplicate database effects; server rejection/conflict is visible and recoverable; two clients converge after create/update/delete and tenant isolation remains enforced.

### 14. pri-c014-aso-memory — Diagnose and reduce the existing ASO replica memory cost

- **Owner / scope:** ASO materializer + PEM; C08. **Priority:** P0.
- **Depends on:** pri-c002, pri-c003, pri-c013. **Decision inputs:** Q7 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reconcile current source with mounted receipts and profile the existing 16203-row workload. Apply measured, bounded memory corrections while preserving SQL/checkpoint atomicity, owner fencing and the experimental gate.
- **Library:** library: cand-003. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Same-fixture browser campaign measures incremental RSS at or below 536870912 bytes and heap at or below 268435456 bytes, unless c003 records an explicit operator-approved replacement contract.
- **Acceptance B:** Worker/DB/graph-copy and cold-fold allocations explain the result; passing behavior survives optimization. If one bounded session cannot close the gate, record findings and split remaining implementation before continuing; do not close this change on diagnosis alone. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.

### 15. pri-c015-aso-protected-proof — Qualify the protected shape-to-SQL-to-PEM path

- **Owner / scope:** ASO + Gate/Fabric + PEM; C08. **Priority:** P0.
- **Depends on:** pri-c005, pri-c013, pri-c014. **Decision inputs:** Q2, Q7 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Prove current-source protected replication using the existing shape feed and materializer, independently of generic entity watching. Cover every selected browser/native topology and approved schema/egress boundary.
- **Library:** library: cand-003. Reuse and evidence obligations are copied into design.md.
- **Acceptance A:** Crash before SQL checkpoint rolls back rows/checkpoints; crash after SQL commit before PEM publication recovers coherently only after current authorization. Logout/practice switch fences stale owners and clears the old graph.
- **Acceptance B:** Interrupted shape bodies, expiry/refetch, revocation timing, direct-backend isolation and forbidden/local-only field egress pass for each selected topology; no competing generic-watch writer handles the same clinical rows. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.

### 16. pri-c017-hosted-media — Qualify hosted media on its selected profile

- **Owner / scope:** Fabric LiveKit + deployment; C11. **Priority:** P1.
- **Depends on:** pri-c002, pri-c003, pri-c004, pri-c005. **Decision inputs:** Q1, Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reuse researched LiveKit integration; prove real two-participant decoded audio/video and grant lifetime on the selected server version. Qualify relay only if advertised.
- **Acceptance A:** Two clients show progressing decoded audio/video across late join and reconnect; wrong room/tenant/subject is denied and revoked participants stop under the accepted lifetime contract.
- **Acceptance B:** Any advertised cross-node relay passes a two-gateway campaign; absent configurations fail visibly. Local protocol fixtures alone cannot certify an untested production service version.

### 17. pri-c018-matrix-recovery — Qualify Matrix directions and restart cursors

- **Owner / scope:** Fabric Matrix bridge; C12. **Priority:** P1.
- **Depends on:** pri-c002, pri-c003, pri-c004, pri-c005, pri-c007. **Decision inputs:** Q1, Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reuse actual Matrix sync/send code. Repair demonstrated direction/cursor/echo defects and stale stub logs; qualify the accepted Tuwunel/Matrix version.
- **Acceptance A:** Inbound and outbound events reach intended rooms/channels after bridge restart with bounded deduplication and durable cursor continuity.
- **Acceptance B:** Echo-loop suppression, cross-tenant denial, target outage and unconfigured direction errors are asserted against the actual selected local server version.

### 18. pri-c019-atproto-recovery — Qualify ATProto ingestion and outbound recovery

- **Owner / scope:** Fabric ATProto bridge; C12. **Priority:** P1.
- **Depends on:** pri-c002, pri-c003, pri-c004, pri-c005, pri-c007. **Decision inputs:** Q1, Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reuse existing firehose and configured outbound implementations. Prove stable cursor/channel identity, authorization and retry behavior against selected Tranquil/PDS versions.
- **Acceptance A:** Supported directions resume after disconnect/process restart without missed acknowledged events or unbounded duplication.
- **Acceptance B:** Outbound retries, self-echo, tenant/channel routing and unavailable/unconfigured targets have explicit assertions; faithful protocol mocks are labeled and cannot substitute for target-version qualification.

### 19. pri-c020-sovereign-decode — Close sovereign media decode and protected fan-out

- **Owner / scope:** Fabric str0m media; C13. **Priority:** P1.
- **Depends on:** pri-c002, pri-c003, pri-c004, pri-c005. **Decision inputs:** Q1, Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Reproduce the zero-decoded-frame failure in a locally controlled Linux ICE/TURN topology. Trace room/track/keyframe routing, apply the smallest demonstrated correction and prove room-scoped protected egress.
- **Acceptance A:** Repeated two-peer campaigns show increasing receiver framesDecoded plus audio progress, including late join/reconnect and multiple rooms; ICE/RTP alone is insufficient.
- **Acceptance B:** Revocation/expiry removes protected fan-out within c003's accepted bound; wrong-room/tenant traffic is denied and task/socket counts return to baseline. Keep production mode gated until proof.

### 20. pri-c021-admin-auth — Deliver the accepted admin authentication profile

- **Owner / scope:** Fabric admin + Gate; C14. **Priority:** P1.
- **Depends on:** pri-c002, pri-c003, pri-c004, pri-c005. **Decision inputs:** Q1, Q2 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** M; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Implement only the admin profile accepted in c003: either a deliberately restricted operator tool or the separately accepted interactive login design. Reuse the actual Gate/IdP surface without assuming an unrelated identity provider.
- **Acceptance A:** Selected admin login/token entry, expiry, logout and unauthorized access behavior work against real local identity services.
- **Acceptance B:** Interactive-login claims require the actual redirect/session flow; a documented operator-only profile is never reported as delivered interactive login.

### 21. pri-c022-platform-parity — Qualify required native and generated SDK transports

- **Owner / scope:** Fabric SDK/FFI + consumer owners; C14. **Priority:** P1.
- **Depends on:** pri-c003, pri-c006, pri-c012, pri-c013. **Decision inputs:** Q3 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Expand the working c012-c013 browser/Node transport into the selected native/generated SDK surfaces using pinned or explicitly superseded ADR-003 tooling. Address Dart async transport only through a demonstrated compatible approach; qualify every required install/runtime/device surface.
- **Acceptance A:** Each required platform installs the packaged artifact and performs authenticated publish/watch/reconnect; supported offline outbox behavior survives process replacement.
- **Acceptance B:** Dart async remains blocked until a real transport proof passes; unsupported platforms remain excluded with explicit reasons, without claiming full phase closure or parity.

### 22. pri-c016-operations-recovery — Qualify capacity, recovery and rotation per profile

- **Owner / scope:** Deployment + Fabric operations; C10. **Priority:** P0.
- **Depends on:** pri-c003, pri-c004, pri-c005, pri-c007, pri-c008, pri-c009, pri-c010, pri-c011, pri-c012, pri-c013, pri-c014, pri-c015, pri-c017, pri-c018, pri-c019, pri-c020, pri-c021, pri-c022. **Decision inputs:** Q2, Q6 via c003.
- **Recommended agent:** Codex; operator resolves recorded decisions. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Execute load and failure campaigns at accepted profile targets; cover broker/WAL/checkpoint lag, backup/restore, rolling replacement, secrets/certificate rotation, drains and no-progress alerts. Final qualification cannot close before every selected profile implementation and client surface is complete; early diagnostic campaigns are non-final.
- **Acceptance A:** Local campaigns meet accepted subscription/change-rate/latency/retention/RTO/RPO targets with explicit resource allocation, no required skips and recorded recovery positions.
- **Acceptance B:** Restoration and credential/certificate rotation preserve required authorization and durable state; bounded backpressure/cancellation and alerts prevent false-ready/no-progress operation. Repeat affected receipts after consumer or profile changes.

### 23. pri-c023-release-signoff — Produce source-bound release verdicts for each agreed profile

- **Owner / scope:** Fabric release + all consumer owners; C15. **Priority:** P0.
- **Depends on:** pri-c001, pri-c002, pri-c003, pri-c004, pri-c005, pri-c006, pri-c007, pri-c008, pri-c009, pri-c010, pri-c011, pri-c012, pri-c013, pri-c014, pri-c015, pri-c016, pri-c017, pri-c018, pri-c019, pri-c020, pri-c021, pri-c022. **Decision inputs:** none beyond dependencies.
- **Recommended agent:** Codex. **Est. complexity:** L; **Complexity score:** High; **Model class:** frontier. **Customer value:** HIGH.
- **Details:** Re-audit effective source/configuration, verify all selected profile receipts and publish a dated local GO/NO-GO matrix. Reconcile capability, rollout/rollback and support documentation; retain separate phase, certification and publication statuses.
- **Acceptance A:** Every selected profile has exact repository/diff/lockfile/package/image/config/topology fingerprints, commands, timestamps, exits, scenario counts and artifact hashes; critical/high findings are resolved by an independent reviewer.
- **Acceptance B:** A narrower profile may receive a scoped interim verdict while other capabilities remain blocked; c023 and the whole phase cannot close until all agreed phase goals are satisfied or the operator explicitly accepts scope reduction. No registry publication/deployment is inferred.

## Execution order and shared-file safety

The dependency graph in planned-changes.json is authoritative for ordering.
Revision 3 prioritizes a usable vertical slice over platform breadth. Read
proposal/design/tasks and the relevant repository instructions before each
slice; completed checkboxes remain authoritative.

- First independent source slice: c001. c003 decision/research work can occur
  alongside c001, but cannot close without operator answers or explicit delegation.
- c002 establishes local fixtures after c001. c004–c005 qualify profile/authority.
- Prototype chain: c006 → c007 → c008 → c009 → c010 → c011 → c012 → c013.
  At every boundary, keep the last runnable profile available: LISTEN remains
  the Forge rollback source and the c012 online path remains available while
  c013 adds offline durability.
- ASO follows c013: c014 → c015. It reuses the now-working local sync and PEM
  seams while preserving the existing protected-shape authority boundary.
- Operations c016 is ordered after all selected implementation/client changes.
  Its final receipts cannot close before those dependencies; any earlier campaign
  is diagnostic. Changed source/configuration invalidates affected receipts.
- Media c017/c020, federation c018/c019, admin c021 and native c022 keep separate
  gates after their listed dependencies. No first-release priority is invented.
- c023 closes the whole phase only after every agreed obligation is satisfied.
  Its interim profile verdict may be written earlier with outstanding dependencies
  explicitly shown; an interim GO never completes c023 or the phase.

These are dependency waves, not permission to edit shared files concurrently.
Gateway composition, domain/ports, shared manifests and SDK generation need
serialized integration or isolated worktrees with explicit merge ownership.
Sibling repository work belongs to its own checkout/PR and follows its own rules;
central OpenSpec proposals track the cross-project receipts and commit references.

## Required verification and release gates

The existing four data-path candidate evaluations do not qualify all dependencies.
c003 supplies missing research before dependent designs become implementation-ready.
Retain all phase goals unless the operator explicitly accepts a scope reduction.

| Profile/capability | Minimum relevant proof | Verdict constraints |
|---|---|---|
| Prototype realtime data: Forge + PGlite + Electric + PEM | c001–c013; one complete local user-flow receipt before broader qualification | Forge owns upstream writes; Electric/Fabric owns canonical downstream reconciliation; no direct-database client shortcut |
| Fabric CRDT persistence | Deferred from the prototype critical path to a later phase | Persistent CRDT adapters are not represented as complete by the PGlite outbox work |
| ASO protected replica | c001–c005, c014–c015; applicable c016 recovery/rotation proof | Memory, atomic checkpoint, owner fencing, approved schema and deployment isolation all pass; no generic-watch dependency |
| Hosted media | c001–c005, c017; applicable operations proof | Positive decode and intended grant lifetime; cross-node only if separately proven |
| Matrix / ATProto | c001–c007, c018/c019; applicable operations proof | Actual target-server version and supported direction; local protocol mocks alone insufficient |
| Sovereign media | c001–c005, c020; applicable operations proof | Increasing decoded frames/audio; intended-room-only delivery and bounded revocation |
| Admin / SDK platforms | c021/c022 plus their prerequisites | Each claimed login/install/transport surface has actual proof |

Every release receipt names repository commits AND dirty-diff hashes, lockfiles,
generated-contract hashes, package/image digests, effective feature/configuration
fingerprint, local topology/resources, exact command, time, exit status, expected
scenario count, required skips (must be zero), and artifact hashes. Missing fields
or stale affected inputs keep the gate open. Prior ASO behavior receipts may be
reused only after source/scope reconciliation; its failed memory gate remains failed.

Load figures from revision 1 (1000 subscriptions, 100 changes/s, p95 1 s, p99 3 s,
60 minutes) remain proposed starting points only. c003 must fix numerical targets
and partition/retention/acknowledgement assumptions before c016 executes a campaign.
A test with no declared threshold cannot produce a production capacity verdict.

## OpenSpec commands and next action

Each listed change is emitted under openspec/changes/<id>/ with proposal.md,
design.md, tasks.md and a scenario-bearing spec delta after plan review. Creation
equivalent: `/opsx:new <id>`; do not recreate directories already emitted.

Next source action: continue `/opsx:apply pri-c011-forge-watch` at task 2 and
finish the runnable Forge lane before beginning c012. The operator has already
authorized execution; this revision does not pause at planning.

## Review and state

Plan adversarial review: two rounds, final reviewed verdict BLOCK (one critical,
one warning). Both reports passed the anti-theater screen. The final corrections
below have not received a third independent review. Assess's review BLOCK and Analyze's bounded
coverage remain historical inputs, not inherited approval. c003 includes focused
resolution of the Assess provenance issue. Plan review evaluates ordering and
testability; it does not certify implementation or deployment.

Waypoint files identify c011 task 2 as the next action and retain actual
implementation/certification/publication counters. Runtime migration and legacy
ambiguity repair remain outside this invocation.


## Unresolved independent-review status

The skill's two-round limit was reached. Retain the second-round CRITICAL verbatim:

> pri-c003 is circularly blocked by the decisions it is supposed to resolve.

Final correction: c003 has no pre-start decision inputs. Q1–Q7 are explicit
closure outputs; research/facilitation may start immediately. Dependent product
implementation remains gated. The second-round warning about c011 client
ordering is corrected by assigning tonic Rust generation/export/compile proof
to c010, before Forge, while non-Rust package generation remains in c012.

Round-1 corrections moved final c016 qualification after the implementations it
certifies, added TLS/bootstrap boundary assertions and required ASO touched-file
size compliance. All reviewed snapshots are preserved in review/plan/.
These corrections are concrete but not independently re-vetted. Status remains
review BLOCK / re-review pending, not PASS. A focused independent review of the
corrected plan is required before c003 closes and dependent execution is released;
c001 and c002 remain independently scoped foundational work.
