# Source and runtime inventory — pri-c003

Captured: 2026-09-15. This inventory identifies the source used to make c003
decisions. It is not a release receipt; c023 must bind the final commits,
packages, generated outputs, images, configuration and topology again.

## Repositories

| Project | Revision inspected | State used by c003 |
|---|---|---|
| Fabric | `52ab9b3874abbe35bbca73720d85578f3db70943` | branch `codex/production-readiness-integration`; pre-existing shape edits excluded |
| Gate | `97d65437c937a285fb000f5e42fddc8af4ff9934` | clean |
| Forge | `dc313be3a044c65b05d845a9c34350bf5ca3ca3e` | clean |
| PEM | `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565` | clean |
| ASO | `d95242542060d1e19a6efe9143f57c015d6162f4` | dirty user checkout, inspected read-only; later edits require an isolated worktree |

The exact status hashes and starting document hashes are in `baseline.md`.

## Fabric locked packages

| Surface | Lock identity |
|---|---|
| authorization policy | `cedar-policy 4.11.1`, checksum `716a5103f447735b4cef15df847dc2a7ed8752e8a95febab5a6bfe1812054e43` |
| broker | `iggy 0.6.203`, git commit `d34b9c96ad5a15334e06040d68fd7512beeba4c8` |
| JWT | `jsonwebtoken 9.3.1` and transitive `10.4.0` |
| hosted media API | `livekit-api 0.5.2`, checksum `394e625d8da0d89f811b7b30beb741df18e75077a5226176cec708957b07a1fa` |
| CRDT | `loro 1.13.1`, checksum `7bd1b63cd2f0cf66acbbb3191c41feaff03c94c3f6c6bb1e61d2ab3062c301c7` |
| WAL | `pg_walstream 0.6.3`, checksum `54e3661e27a088609a68a3ec7885705ccf01d2bb9f211f00c27302009d69d59a` |
| local operation store | `redb 4.1.0`, checksum `8e925444704b5f17d32bf42f5b6e2df050bceebc3dcd6e71cc73dafe8092e839` |
| sovereign WebRTC | `str0m 0.21.0`, checksum `fed3d9290b349b6da18fd640d81a950e4b256dba80e36982be61ae6ef98fb025` |
| external durable store | `surrealdb 3.1.5`, checksum `81ee3110fe3ab8172eb8c135c96ed2d57c7470cdf1ad732d176db95a92c9faab` |
| native bindings | `uniffi 0.31.2`, checksum `46eefd5468602930da46b1f49d3448c6dfc2e81295f93120f23f8174fd70267f` |
| browser bindings | `wasm-bindgen 0.2.125`, checksum `8ddb3f79143bced6de84270411622a2699cee572fc0875aeaf1e7867cf9fca1a` |
| TypeScript RPC | lock resolves `@connectrpc/connect 1.7.0`, `@connectrpc/connect-web 1.7.0`, `@bufbuild/protobuf 1.10.1` |

## Runtime image/config state

| Project/profile | Current identity | Qualification consequence |
|---|---|---|
| Fabric integration Iggy | `iggyrs/iggy:latest`; c002 receipt captured the pulled digest | c004/c007 must replace the mutable tag with the accepted server digest |
| Fabric Keto | `oryd/keto:v0.12` | exact patch/digest and migration compatibility still required |
| Fabric SurrealDB | mutable `latest` | cannot enter a production profile until exact server/client selection |
| Fabric Postgres | `postgres:17-alpine` | exact digest, backup/restore and replication configuration required |
| Gate Postgres | `postgres:16-alpine` | exact digest and backup/restore required |
| Gate Kratos | `oryd/kratos:v1.2` | minor-range tag is not an immutable release identity |
| Gate Hydra fixtures | `oryd/hydra:v2.2.0` | test-only currently; production authority decision belongs to c004/c005 |
| Forge Postgres | local `images/postgres18/Dockerfile` | final built image digest and extension inputs required |
| Forge ingress/ops | `caddy:2-alpine`, Prometheus `v3.0.0`, Alertmanager `v0.27.0` | mutable Caddy base and exact observability images need c004/c016 receipts |
| ASO browser store | PGlite 0.5.8 base tarball SHA-256 `f4818a049c2e017209433b00518760dbd8694d20c88fdc59b3bc2aea299f635f` | identity is pinned; memory gate remains failed |

## ASO restricted-shape source contract

Source: `docker/frf/shape-catalog.json`, `web/src/shared/sync/electric-shapes.ts`
and `web/src/shared/sync/pglite-schema.ts` at the ASO revision above.

| Table | Key | Tenant/scoping | Permitted replica columns |
|---|---|---|---|
| `aso.cases` | `id uuid` | `practice_id uuid` | id, practice_id, status, gate_affirmed_at, created_at, updated_at |
| `aso.case_evidence` | `id uuid` | denormalized `practice_id uuid` forced from case | id, practice_id, case_id, policy_criterion_id, state, assessed_at, created_at, updated_at |
| `aso.evidence_citations` | `id uuid` | denormalized `practice_id uuid` forced from case evidence | id, practice_id, case_evidence_id, document_id, page_number, relevance, created_at |
| `aso.documents` | `id uuid` | denormalized `practice_id uuid` forced from patient/case | id, practice_id, document_type_id, case_id, name, effective_date, page_count, content_sha256 |
| `aso.evidence_states` | `key text` | explicit global reference set | key, label, meaning |

The catalog excludes patient identifiers, clinician identifiers, storage URIs,
free clinical text and unbounded JSON. ASO's server-side triggers derive the
practice on child rows because the Electric predicate is flat and cannot join.

## Current adapter behavior that drives later work

- Fabric CDC initializes a local envelope offset at process start, while the
  Iggy adapter returns that payload field rather than a demonstrated durable
  broker partition position.
- Fabric's Matrix `/sync` adapter carries `next_batch` only in process memory.
- Fabric's Jetstream adapter reconnects without a persisted/supplied cursor.
- Fabric's hosted-media adapter uses `livekit-api`; it does not yet compose the
  realtime room SDK/event loop needed for received data.
- Fabric's full gateway currently composes the redb operation store in memory,
  despite persistent adapters existing elsewhere.
- Forge currently defaults `FLINT_CHANGE_SOURCE=listen`; the Fabric source
  cannot replace it until c010/c011 parity and rollback pass.
- PEM source packages are ESM. ASO consumes exact prerelease core/react tarballs
  recorded in `versions.toml`; source repository versions alone are not the ASO
  artifact identity.

## Provenance rules carried forward

Current source outranks historical receipts. Each later cross-repository change
must recapture repository revisions and dirty-diff hashes, use isolated
worktrees around foreign changes, and run its repository's local checks. Image
tags, manifest ranges, Git timestamps and comments are inventory clues only;
none independently prove compatibility, security, durability or production
readiness.
