# Accepted release contract — pri-c003

Accepted: 2026-09-15 through the operator's autonomous execution authorization
recorded in `execution.md`. These are the conservative defaults requested there;
later operator direction may supersede them explicitly.

## Q1 — first release

The first releasable scope is the data path shared by Fabric, Gate, Forge, PEM,
and ASO. It includes authenticated database changes, durable replay, entity
projection, the versioned entity-type watch, Forge GraphQL delivery, packaged
TypeScript and Rust clients, PEM graph updates, persistent full-profile CRDT
state, and the ASO restricted replica.

Media, Matrix/ATProto federation, the admin UI, and additional native SDKs keep
their own phase gates. They remain phase goals, but they do not block a
data-profile release verdict.

## Q2 — deployment profiles and consumers

| Profile | Required topology | Required consumers | Release verdict boundary |
|---|---|---|---|
| `full` | Fabric gateway, Gate identity/authorization, Postgres CDC, durable Iggy, persistent projection/checkpoints and persistent CRDT snapshot/operation storage | Forge, PEM, Rust and Node clients | Data and declared offline-sync APIs; media/federation/admin stay disabled until their gates pass |
| `shape-only` | Gate, Fabric HTTP shape facade, Electric, Postgres, browser PGlite | ASO browser through pinned PEM packages | Only catalogued ASO shapes; gRPC, CDC, event, media and federation lanes remain off |

Each profile receives its own configuration, image, readiness, recovery, load,
and authorization receipt. One profile cannot certify the other. Production
secrets, TLS, ingress and durable volumes are mandatory; mutable image tags do
not qualify.

## Q3 — required clients

First-release clients are:

- TypeScript browser/PEM using the actual browser transport;
- Node.js ESM and CommonJS using packed package artifacts; and
- Rust using the generated tonic client.

Swift, Kotlin, Python, Dart, Go and C# remain later platform gates. UniFFI may
continue to generate Swift/Kotlin/Python bindings, but their presence is not a
release claim.

## Q4 — durability and history

The data stream is durable and at least once. A resume cursor can redeliver an
event, so consumers must deduplicate by stable event identity and apply changes
idempotently. Cursor state distinguishes source epoch/LSN, transaction/event
identity, broker partition position, and consumer checkpoint.

History is bounded. The production minimum retention target is 24 hours and
must exceed the measured worst-case restore plus catch-up interval by at least
2x. A cursor outside retained history returns an explicit `resnapshot_required`
condition; it never silently starts at the oldest surviving event. The consumer
then obtains an authorized snapshot and resumes after its snapshot/WAL barrier.

## Q5 — sources, keys, tenants and deletes

### ASO restricted-shape profile

The source catalog at ASO revision
`d95242542060d1e19a6efe9143f57c015d6162f4` is authoritative:

| Shape | Source table | Primary key | Tenant rule |
|---|---|---|---|
| `cases` | `aso.cases` | `id uuid` | `practice_id uuid` |
| `case_evidence` | `aso.case_evidence` | `id uuid` | `practice_id uuid`, server-derived |
| `evidence_citations` | `aso.evidence_citations` | `id uuid` | `practice_id uuid`, server-derived |
| `documents` | `aso.documents` | `id uuid` | `practice_id uuid`, server-derived |
| `evidence_states` | `aso.evidence_states` | `key text` | explicit non-PHI global reference set |

Only the columns in `docker/frf/shape-catalog.json` may cross the facade. The
client cannot choose tables, columns, predicates, or practice scope.

### Full Fabric profile

Production CDC enrollment is an explicit allowlist of schema-qualified tables.
It accepts validated single or composite PostgreSQL primary keys, including
non-UUID keys, and an explicitly configured tenant column. A table without a
primary key, tenant mapping, adequate replica identity, or supported typed-value
mapping fails enrollment. The first acceptance fixture must cover a non-first
key, a composite key, and a non-UUID key.

Deletes deliver an authorized invalidation containing canonical key, entity
type, tenant, event identity and cursor. Historical/old row contents are not
sent. A separately named audit consumer and policy would be required to add old
row data.

## Q6 — scale, retention and recovery objectives

| Objective | Accepted threshold |
|---|---|
| Concurrent active subscriptions | 1,000 |
| Sustained committed source rate | 100 changes/second for 60 minutes |
| Commit-to-authorized-client latency | p95 <= 1 second; p99 <= 3 seconds |
| Broker/history retention | >= 24 hours and >= 2x measured restore + catch-up |
| RPO | 0 acknowledged source commits; duplicate delivery is permitted |
| RTO | <= 15 minutes to healthy service and caught-up required projections |
| Slow-client behavior | bounded buffer, explicit lag/resnapshot error, no silent drop |

The final release receipt must measure these values on the exact selected
images and storage configuration. A failed threshold keeps that profile blocked
and may raise retention, but cannot lower the objective without explicit
operator approval.

## Q7 — ASO resource and revocation bounds

The first ASO qualification target is the browser. Its 16,203-row memory-only
campaign must remain at or below 536,870,912 bytes incremental RSS and
268,435,456 bytes heap. The current 1,016,692,736-byte RSS result remains a
blocking measurement.

Revocation begins at the authoritative ASO membership commit, durable session
denial commit, or verified expiry instant. It ends within 5,000 ms at the last
protected response frame produced by Fabric, or cancellation that prevents the
next frame; the next protected request must be denied. Network transit and
client receipt are outside this server lifetime measure.

## Authorization lifetime by profile and endpoint

| Profile / endpoint | Required checks and lifetime |
|---|---|
| `full`: versioned `WatchEntityType` | verified identity and type/tenant permission before stream admission; object authorization before every protected event; cancel on revocation/expiry and never emit a denied payload |
| `full`: v1 `GetEntity` / `WatchEntity` | verified identity plus Keto object authorization for each read/event; unchanged v1 wire contract |
| `full`: `/ws/v1/subscribe` and publish | verified token and channel authorization at admission; reauthorize protected delivery when authority may have changed; token expiry closes delivery |
| `shape-only`: every `/v1/shape` snapshot/continuation request | verified Gate session, practice membership, catalog authorization and current authority revision; lease/cancellation enforces the five-second final-frame bound |

Authorization service failure is fail closed. Readiness is false when a
required authority endpoint cannot answer. Health-only endpoints expose no
protected data.

## Q8 — KBD authority

Legacy KBD authority remains unchanged. No migration is authorized or required
by c003. The reversible archive named in `execution.md` is retained only as a
recovery artifact.

## Compatibility and rollback

`proto/flint/v1` stays byte-identical. The entity-type watch is introduced in a
new versioned namespace. Rollback disables the new route/profile and restores
the prior binaries without rewriting v1 data. Persistent cursor/projection
formats require an explicit version and backward reader or a documented
resnapshot path before rollout.
