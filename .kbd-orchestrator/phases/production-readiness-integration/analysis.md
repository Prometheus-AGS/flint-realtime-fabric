# ANALYSIS: production-readiness-integration

Date: 2026-09-15 · Mode: stack specified · Implementation: 0/0 registered changes

## Conclusion

Retain the established stack as the research baseline. This is a **partial,
bounded analysis of the data path**, not completed research for every phase goal.
Provisionally adapt four existing components: Iggy, pg_walstream,
PGlite/the ASO materializer, and Connect-ES/generated SDKs. Build the missing
cross-project contracts, composition and qualification around them. No new
dependency or major-version upgrade is selected by this analysis.
The broader phase goals remain in scope but cannot be fully specified or planned
from these four candidate evaluations alone; see the research backlog below.

**Provisional pending operator answers:** the first release's capability set,
deployment topology, client platforms, delivery semantics, source schema/tenant
model, operational targets and ASO platform priority. Eight questions were sent
through the task's question UI. Defaults shown there are recommendations, not
answers. No dependent contract choice is frozen here.

The assessment remains NOT READY. This analysis adds recovery findings; it does
not repeat the previous build checks or claim new runtime evidence. The bounded research artifacts can be handed off independently of canonical KBD
stage recording. Full-phase research and clarification remain incomplete.

## Evidence and research scope

- Baseline and cross-project findings: [assessment.md](assessment.md), its source
  inventory and [source excerpts](evidence/review-source-excerpts.json).
- Current research responses: [analyze-research.json](evidence/analyze-research.json).
- New pinned-source evidence: [analyze-source-excerpts.json](evidence/analyze-source-excerpts.json).
- Machine decisions: [library-candidates.json](library-candidates.json).
- Tier 1: four GitHub repository searches. Exact upstream matches were selected
  manually; incidental results were excluded. Upstream activity is a maintenance
  signal, not evidence of the pinned fork's behavior.
- Tier 2: four Context7 resolutions, three documentation queries and one failed
  primary-doc open, reaching the eight-query cap. Context7 had no pg_walstream
  match. The installed 0.6.3 source supplied limited version-specific evidence.
- Tier 3: three npm lookups and two failed crates.io requests (TLS trust error,
  then HTTP 403). No verification was bypassed. WAL registry currency is unknown.
- Tier 4: not used. Broad stack comparisons would not resolve the concrete
  adapter defects. Research stopped within the 20-minute ceiling; candidate
  API confidence is deliberately limited where exact-version docs were absent.

## Candidate decisions

| Candidate | Verdict | Scope supplied by dependency | Work Fabric/consumer projects still own |
|---|---|---|---|
| cand-001: pinned Iggy fork | Adapt | Broker storage and offset polling | Correct offset mapping, explicit acknowledgements, retention/resync, durable entity projection and authorization |
| cand-002: pg_walstream 0.6.3 | Adapt | Logical replication parsing and relation metadata | Commit boundary, tenant/schema/key/value mapping, poison-event policy and durable source checkpoint |
| cand-003: PGlite + ASO materializer | Adapt | Existing SQL replica, worker/database lifecycle | Measured memory reduction, current-source receipts and platform qualification |
| cand-004: Connect-ES 1.x + generation | Adapt | Existing RPC transport and generation pipeline | New frozen proto version, package exports and real authenticated consumer tests |

No numeric coverage estimates are assigned: a library API's presence does not
measure the percentage of an integration requirement that works.

### Iggy: reuse storage, repair cursor and acknowledgement semantics

Current upstream documents explicit polling offsets and stored consumer offsets.
[Iggy schema](https://iggy.apache.org/docs/server/schema).
The pinned fork is `d34b9c96ad5a15334e06040d68fd7512beeba4c8`.

New static findings, captured in the pinned-source evidence:

1. `frf-postgres-cdc/src/consumer.rs` initializes its envelope counter to
   `Offset::BEGINNING` on every run and increments it locally after publish.
2. `frf-broker-iggy/src/broker.rs` serializes that supplied envelope and returns
   `envelope.offset`; consumption deserializes the payload without substituting
   Iggy's message offset. Therefore the exposed counter is not demonstrated to
   be a stable broker replay position, especially after CDC restart.
3. The adapter builds its consumer without overriding auto-commit. The pinned
   builder defaults to `IntervalOrWhen(ONE_SECOND, PollingMessages)` and
   `allow_replay: false`. Explicit ack methods alone do not establish that an
   application commit controls broker progress.

**Required design:** distinguish source LSN, event identity, partition position
and subscriber checkpoint. Define cursor inclusivity, partition/source epoch,
retention expiry and duplicate handling. Broker acknowledgement must follow the
chosen durable handoff; consumer checkpoint must follow its durable application
commit. Prove actual fork replay behavior, server persistence/flush settings,
consumer cancellation and bounded slow-client buffers with local crash cases.
These are risks established by composition inspection, not observed data loss.

The existing v1 EntityService needs a populated persistent projection through an
EntityStore adapter. Its population/checkpoint capability needs a separate port
and adapter if the current read/watch port cannot express it; preserve one port
per adapter and compose only in the gateway. Define initial snapshot plus WAL
barrier and projector catch-up before GetEntity readiness. Broker persistence
alone does not populate the currently empty read store.

### WAL: preserve metadata and committed transaction boundaries

The installed parser has relation namespace, column information and
replica-identity key flags. Reuse it. Those flags are not necessarily the
application primary key; prove initial metadata exposure through the selected
event API or obtain a validated catalog mapping. FRF currently synthesizes the
first-column UUID assumption and ignores transaction/control events.

Define handling for composite/non-UUID keys, schema qualification, typed values,
unchanged TOAST, nullable fields, schema changes and delete identity. Reject
unsupported tables at enrollment instead of silently misidentifying events.
Determine transaction publication and checkpointing across multi-row commits;
advancing applied LSN after publish must mean the chosen durable guarantee has
actually been met. A decode error cannot be skipped and then hidden by a later
checkpoint if the selected contract promises every committed change.

Alternative CDC services were not evaluated in this bounded pass. Keeping the
existing parser is a provisional fit-based recommendation under the fixed stack,
not a comparative verdict against unevaluated services. Revisit alternatives if
maintenance or required semantics disqualify the current parser. **Conditional decision:** maintenance/security verification is required before
accepting pg_walstream for production. The adapt verdict means a provisional
design preference, not completed dependency qualification. Exact maintenance
health remains unknown.

### ASO: reuse materialization and diagnose its measured memory failure

Existing mounted receipts already cover SQL/checkpoint atomicity and owner fences.
The assessed incremental RSS is 1,008,877,568 bytes against 536,870,912 allowed;
heap passes. Preserve the experimental adoption gate. Reconcile receipt hashes
with the active ASO working tree before reusing any result.

PGlite provides database shutdown and worker integration. Neither establishes
that process RSS will fall below ASO's limit. [PGlite API](https://github.com/electric-sql/pglite/blob/main/docs/docs/api.md),
[worker lifecycle](https://github.com/electric-sql/pglite/blob/main/docs/docs/multi-tab-worker.md).
Registry latest and ASO's declared range both identify the 0.5.8 line; resolve
the actual lock/install identity for qualification.

Profile database/WASM allocation, simultaneous workers, SQL/result copies,
graph construction and cold-fold batch sizes against the same 16,203-row fixture.
Preserve transaction/checkpoint atomicity, owner fencing and authorized graph
publication during optimization. A worker move, dependency upgrade or storage
replacement is a hypothesis requiring measurement, not a promised solution.

Correct the draft plan's revocation wording now at analysis level: retain the
existing five-second bound from authoritative ASO commit/expiry to the last
server-produced protected frame or cancellation preventing the next frame,
followed by new-request denial. Network transit and client receipt are excluded.
Direct Kratos revocation uses the documented observation/ASO-denial start point.
A stronger client-receipt requirement would need a new contract and feasibility
analysis. The existing requirement is not silently weakened.

### SDKs: distinguish protocol version from library version

New RPCs go in a versioned proto namespace; frozen v1 files remain unchanged.
Generate bindings after that contract is accepted. A v2 Fabric RPC does not
require upgrading Connect-ES to v2.

Current upstream migration guidance changes client construction and protobuf
generation. Keep ADR-003's toolchain until explicitly superseded; current docs
are not an exact-version API proof. [Connect-ES migration](https://github.com/connectrpc/connect-es/blob/main/MIGRATING.md).
Build and consume packed artifacts through each advertised module entry. If CJS
is required, emit and execute a real compatible CJS artifact; otherwise changing
the advertised support contract is an explicit compatibility decision.

PEM must consume a real gateway stream using the published payload mapping.
Validate SQL mutation → CDC → broker → SDK → graph, including reconnect,
authorization changes, duplicates and deletion. Synthetic loopback stays useful
but cannot certify this route. Prove actual browser transport and cancellation
through the selected gateway/ingress, not only generated type compatibility.

## Integration ownership and dependency order

| Work packages | Owner and required outcome |
|---|---|
| C01–C03 | FRF + Gate: build/policy corrections, explicit full versus restricted-shape profiles, reachable endpoint authorization and truthful readiness |
| C04–C05 | FRF with Forge/PEM contract review: versioned watch, committed ingestion, durable replay and v1 read/watch population |
| C06 | Forge: real Fabric adapter, authorized projected rows, safe delete/invalidation and overflow reconstruction; retain LISTEN until real parity passes |
| C07 | FRF SDK + PEM: package artifacts, canonical payload mapping, actual network delivery and declared-platform matrix |
| C08 | ASO + Gate/FRF + PEM: reuse replica implementation, source-bound memory/recovery/fencing/isolation qualification |
| C09–C10 | Integration/operations owners: non-vacuous local composed-stack tests, load, outage, restore, rotation and restart receipts |
| C11–C14 | Existing media/federation/platform owners: selected deployment/platform proofs, sovereign decoded frames and bridge cursor recovery |
| C15 | Release owner: per-profile source/package/image-bound signoff; no blanket readiness from historical counters |

The existing 15-package plan remains a proposal. Its next revision must include
the new broker cursor/autocommit findings and corrected revocation boundary.
Contracts precede implementation; producer/consumer alignment precedes release.
ASO memory profiling can proceed independently of generic entity-watch work.
Do not substitute generic entity watching for ASO's protected shape authority.

Generic exposed entity routes retain the required Keto object authorization and
Forge RLS projection. Restricted shape grants retain their separate Gate/ASO
authority contract. Verified identity alone must not become a substitute for
per-object authorization. Deletion and historical payload delivery require an
explicit authorization model; prefer authorized invalidation when a current-row
RLS query cannot establish historical visibility. An audit stream is a separate
consumer requirement, not permission to expose old rows to every UI subscriber.

Every runtime acceptance check runs locally against the composed stack. CI may
build/lint/typecheck/format/package only. Missing services and empty streams are
not passing evidence. No source, dependency, deployment or registry mutation was
performed in this analysis.

## Required research backlog beyond the four candidates

The eight-query documentation cap ended this research pass. The following goals
are **excluded from completed candidate coverage**, not removed from the phase.
Their work-package rows above preserve assessed obligations and ownership only;
they are not build-versus-adopt verdicts. Do not register a full executable plan
for these surfaces using this artifact alone. Continue focused research after
operator scope answers, in an explicitly bounded follow-up pass.

| Goals / gaps | Existing stack to evaluate | Evidence still required before specification/plan freeze |
|---|---|---|
| G2/G3/G9; F05/F06/F11 | Gate, Kratos, Keto, Cedar, restricted shape grants | Pinned identity/authz/policy API and maintenance fit, route/profile mapping, revocation/rotation boundaries and selected deployment authority |
| G2/G7; F06/F10 | Existing Compose/SSR manifests, ingress, Iggy and SurrealDB persistence, redb | Pinned persistence/backup/restore semantics, durable gateway composition, real dependency health, deployable secrets/TLS/network configuration |
| G8; F10 | LiveKit and str0m | Exact-version supported transports, hosted target setup, frame-decode gap cause, lifetime authorization and current local media receipts |
| G8; F10 | Tuwunel/Matrix and Tranquil/ATProto bridges | Pinned API/maintenance fit, cursor durability, restart replay, echo suppression and target interoperability |
| G5; F08/F10 | UniFFI, Dart generator, Go/C#/Swift/Kotlin SDK builds | Required-platform selection, pinned generator support, async transport feasibility, package/install/network acceptance matrix |
| G9 | Selected release images and all retained dependencies | Maintenance/security qualification and digest-bound release evidence; a repo update timestamp is insufficient |

Build/MSRV/CI-policy defects (F02/F09) are concrete corrections supported by the
assessment, not library-selection questions. They may be planned from that
source; no claim that this research independently revalidated them is made.
No completed coverage is claimed for F10 as a whole: only its broker recovery
concerns were investigated here. Snapshot/CRDT persistence still needs the
separate evaluation above.

## Open questions and specification blockers

| ID | Question sent to operator | Decision it blocks |
|---|---|---|
| Q1 | First-release capability set: data, hosted media, or all planes? | Release grouping and which C11–C14 gates are required first |
| Q2 | Target deployment(s), application consumers and existing infrastructure? | Profile composition, isolation, operations and image qualification |
| Q3 | Required client platforms and Node module formats? | SDK acceptance matrix and native/Dart scope |
| Q4 | Durable changes, latest-state recovery, or indefinite history? | Replay contract, snapshot/resync behavior and retention strategy |
| Q5 | Tenant/database topology, schemas, key shapes and deleted-row audit needs? | Enrollment mapping, authorization, payload and tombstone contract |
| Q6 | Subscriptions, change rate, latency, retention, RTO and RPO? | Capacity, durability and recovery acceptance thresholds |
| Q7 | Preserve ASO RSS/revocation bounds; browser first or native too? | ASO platform qualification and any explicitly revised requirement |
| Q8 | Keep legacy KBD state, or authorize migration with ambiguity resolution? | Canonical stage recording only; independent analysis can finish |

Status at artifact creation: all eight pending. Q1–Q7 must be answered or clearly
delegated before dependent specification choices are frozen. Existing security
and memory requirements remain in force while answers are pending. Q8's default
is to leave authority unchanged; no migration is inferred from invoking Analyze.

## Review status

Two isolated gpt-5.5 review rounds ran against gpt-6-produced artifacts.
Round 1: BLOCK, one critical scope-coverage finding and one WAL-maintenance warning.
The revision explicitly limited completed research to the data path, added the
required full-phase research backlog and made WAL adoption conditional.
Round 2: PASS, zero critical and two warnings: unevaluated alternative CDC services
must not be ruled out, and machine gap coverage must match its stated scope.
Both reports passed the anti-theater screen. Final wording now states that
alternative CDC services were not evaluated; F10 was removed from candidate and
build-item gap identifiers while remaining in the required research backlog.
Those final warning corrections were not sent through a third review.
The exact judged packets and reports are preserved under review/analyze/.

The preceding Assess review remains BLOCK after its separate two-round cap;
missing Forge provenance was subsequently attached but not re-vetted. Analyze's
PASS does not retroactively change that verdict or certify production readiness.
Eight operator answers and the explicit follow-up research remain pending.

## Execute update — c003 accepted contract and completed research

The operator subsequently authorized autonomous execution and conservative
defaults in `execution.md`. Q1–Q7 are now resolved by the source-bound contract
in `evidence/pri-c003-scope-research-contract/release-contract.md`; Q8 retains
legacy KBD authority and no migration. Dependent changes must use that contract
until explicit operator direction supersedes it.

The previously excluded research backlog is now covered in
`dependency-qualification.md` and `source-inventory.md`. The result is not a
blanket dependency approval. A current RustSec scan found eight vulnerabilities
plus unmaintained, unsound and yanked packages in the Fabric lock. The revised
proposal retains the architectural adapters while assigning exact-version,
security-closure and compatibility work to their owning later changes. Mutable
deployment tags and historical source timestamps remain insufficient evidence.

The first release is the Fabric/Gate/Forge/PEM/ASO data path, with independent
`full` and `shape-only` verdicts. It requires browser/PEM, Node ESM/CJS and Rust;
at-least-once replay with explicit resnapshot after retention; the enumerated
ASO shape catalog; validated schema/key/tenant enrollment; authorized delete
invalidations; 1,000 subscriptions and 100 changes/s for 60 minutes; p95 <= 1 s,
p99 <= 3 s; minimum 24-hour retention; zero acknowledged-commit RPO; 15-minute
RTO; ASO RSS <= 512 MiB, heap <= 256 MiB and server final-frame revocation <= 5 s.

The focused c003 independent review supersedes only the unresolved provenance
warning identified by Assess/Plan. It cannot turn their broader historical
BLOCK verdicts into production certification; every later implementation and
release gate remains required.
