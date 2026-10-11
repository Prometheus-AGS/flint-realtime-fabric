# Baseline — pri-c015-aso-protected-proof

Date: 2026-09-17  
Task: 1 of 5  
Result: **Passed — task 2 may begin**

## Accepted scope

The c003 release contract selects one ASO topology for the first release:
`shape-only` using Gate, Fabric HTTP shape facade, Electric, PostgreSQL, browser
PGlite, and pinned PEM packages. Native replica code exists in the ASO checkout,
but Q2/Q3 defer native platforms, so it is not a selected c015 topology and
cannot substitute for or expand the browser proof.

Only catalogued ASO shapes and the columns in `docker/frf/shape-catalog.json`
may cross the facade. The client may not select tables, columns, predicates, or
practice scope. Revocation must end protected server delivery within 5,000 ms.
Release contract SHA-256: `29752a3109a4602a1edcf14e72be70b6f5b43270ba7ea165b3de9ab148c857bd`.

## Repository baseline

| Repository | Branch | HEAD | Dirty | Scoped files |
|---|---|---|---|---:|
| Fabric | `codex/production-readiness-integration` | `ae26f45d8afa037677004cd118b0571b8b552131` | true | 8 |
| ASO | `main` | `d95242542060d1e19a6efe9143f57c015d6162f4` | true | 46 |
| Gate | `main` | `77914a2ad2e3cd57f12fd9131fa8cdddf625f68e` | true | 11 |

Fabric's c015 affected paths are clean. ASO has 42
tracked/untracked entries under `web/src/shared/sync/`. Gate also has pre-existing
tracked changes, including `aso_replica_grant.rs`. c015 must record exact content
hashes and isolate any new delta; it must not overwrite or claim unrelated work.
`baseline.json` binds full status/diff/untracked hashes plus each scoped file.

## Dependency verification

| Dependency | Ledger | Runtime result | Receipt SHA-256 |
|---|---|---|---|
| `pri-c005-authority-lifetime` | COMPLETE | 7/7 scenarios, exit 0; protected shape timing max 1,500 ms | `618b43a8dcf46004028df24687f8ea4df3d5decab43dbe074d8905afe71db45e` |
| `pri-c014-aso-memory` | COMPLETE | PASS; 31/31 bound ASO inputs still current | `9e4587429e79372c948eb14b56e3efe83461359b9e244637ae1d400c792891e8` |

Both required dependencies are complete. c014's browser campaign proves the
16,203-row selected topology remains within its memory limits and preserves all
13 durability/fencing checks. c005 proves fail-closed authority lifetime and the
five-second protected-response bound on its source-bound fixture.

## Existing coverage and remaining proof

Existing source and receipts cover SQL/checkpoint rollback, recovery after SQL
commit before PEM publication, owner fencing, authority failure, malformed and
forbidden shape responses, must-refetch behavior, catalog-only fields, and
projecting over-wide rows onto approved PEM fields.

c015 still needs one current-source, source-bound protected campaign that joins
those behaviors across the selected browser topology. It must add observed
interrupted-body behavior, practice switch/logout graph clearing, current
revocation timing, direct-backend denial, forbidden/local-only egress denial,
and proof that generic watch is not a competing writer for the clinical tables.

## File-size boundary

Pre-existing oversized ASO files in scope remain untouched unless split:

| File | Lines |
|---|---:|
| `web/src/shared/sync/replica-runtime.test.ts` | 659 |
| `web/src/shared/sync/replica-runtime.ts` | 524 |


All c015-modified hand-authored files must remain at or below 500 lines.

## Machine binding

`baseline.json` SHA-256: `65c7c41c6ecb81e0118190b76c287e43f2cf1327eeaa33b0ebb25182114a23a9`
