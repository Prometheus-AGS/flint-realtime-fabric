# Combined c011-c013 full-stack campaign receipt

Captured: 2026-09-17T03:02:17Z

Verdict: **PASS**

This is the shared source-bound receipt for `pri-c011-forge-watch`,
`pri-c012-sdk-pem-network`, and `pri-c013-sync-persistence`. All behavioral
checks ran locally against the composed stack. No CI test result is used.

## Source binding

| Repository | Branch | HEAD | Candidate diff SHA-256 |
|---|---|---|---|
| flint-realtime-fabric | `codex/production-readiness-integration` | `52a655cbe8f8f701e6d56d50457a55b9b33a9a95` | `9826fe0516249c777699d942d593bf5f184b1a6b745e16acb0d106b448def3f4` |
| flint-forge | `codex/production-readiness-integration` | `445a121ad73ed0ba3b6dfafe68bc61ec85cc6b65` | `0bba30cdec925968dd8d07300c374d7c209ef3cf3b4de58751bfe60f13eb6e21` |
| prometheus-entity-sync | `codex/production-readiness-integration` | `a7220cf6ceb38ce01cfbdd90c30f4a9c57dc8389` | `f43ad324b9fa850a543d5fc5734267aab7194c6b00624c5fb94c17fef3a5824e` |

The entity-sync hash covers only the six owned prototype/PGlite files. Other
pre-existing modifications in that checkout are outside this candidate.

Runtime images:

- Fabric: `sha256:99924e2ad018d625685ca6fd9c06c5dc7af5a3fac8008a0d8ba2965ca5d48db5`
- Forge: `sha256:a40126391f6c826ee8c5091f4db5c477f1405766a2db23ff0bb71d21fd4cbbf3`

## Topology

The owned Compose stack ran PostgreSQL, ElectricSQL, Iggy, SurrealDB, the full
Fabric gateway, the Fabric shape facade, Forge, and the local JWT fixture. The
browser prototype used two independent same-tenant PGlite databases plus one
different-tenant database. The final state has all eight services healthy and
Forge configured with `FLINT_CHANGE_SOURCE=fabric`.

## Combined user flow

The browser campaign exercised the full path:

`PGlite optimistic mutation -> durable PGlite outbox -> authenticated Forge REST -> PostgreSQL commit -> Electric/Fabric authorized shape -> PGlite canonical reconciliation -> PEM entity/list observation`

Observed results:

- A create appeared immediately in client A's local PGlite state, committed
  through Forge, and converged into client B through Electric/Fabric and PEM.
- Client B updated the row and both same-tenant clients converged on the
  canonical revision. The different-tenant client never observed the row.
- Forge was stopped, an offline create remained visible in local PGlite with a
  pending outbox entry, and a browser reload preserved both the row and entry.
- After Forge restarted, the stable mutation replayed once, canonical state
  returned through Electric/Fabric, the outbox cleared, and client B converged.
- A duplicate-key create became a visible rejected `409` outbox entry with
  Retry and Discard controls. Discard rebased the local row to canonical state.
- Deletes converged to both same-tenant clients and left the other tenant
  isolated. All campaign rows and outbox entries were removed at cleanup.

The browser can take about 30 seconds to reopen an existing PGlite IndexedDB
after a hard reload. The durable local row and outbox survive, and the UI becomes
usable before Electric's initial shape callback completes. This is a startup
latency rough edge, not an acknowledged-state loss.

## Forge GraphQL and rollback results

The current-source GraphQL campaign passed each required scenario:

| Scenario | Result | Evidence observed |
|---|---|---|
| Authenticated insert | PASS | Exact ID, tenant UUID, label and revision arrived with non-empty event ID/checkpoint. |
| Disconnect and resume | PASS | Update made while disconnected replayed from the prior checkpoint; a subsequent live delete arrived; the database row was absent. |
| Tenant denial | PASS | A tenant-B subscriber saw no tenant-A mutation and did receive its own tenant-B mutation. |
| Primary-key move | PASS | Fabric emitted authorized delete of the old key followed by insert of the new key; final key-only delete arrived. |
| Composite-key move | PASS | `(widget_id, tag)` insert/update, old-key delete, new-key insert and final delete arrived with distinct checkpoints. |
| Truncated LISTEN payload | PASS | A 9,010-character row forced PostgreSQL's PK-only fallback; Forge re-queried the complete row through RLS; the write succeeded. |
| Fabric/LISTEN switch | PASS | Forge became healthy in LISTEN mode, passed reconstruction, returned to Fabric mode, then passed checkpoint recovery again. |

## Retry and conflict boundary

A final idempotency probe used one stable mutation identity:

- first POST: `201`
- exact replay: `201` with the same semantic row
- reused key with a different body: `409`
- database rows for the entity: `1`
- canonical row remained `label = Replay stable`, `revision = 1`

JSON object member order differed between the original and replay response, so
the comparison normalized member order. JSON object order is not semantic.

## Commands and gates

Representative local commands (credentials and bearer values omitted):

```text
./examples/forge-electric-prototype/dev.sh
node /tmp/forge_graphql_probe.mjs
node /tmp/forge_graphql_recovery_probe.mjs
node /tmp/forge_graphql_tenant_probe.mjs
node /tmp/forge_graphql_key_change_probe.mjs
node /tmp/forge_graphql_composite_probe.mjs
node /tmp/forge_idempotency_probe.mjs
docker compose run --rm prototype-configure
FLINT_CHANGE_SOURCE=listen docker compose up -d --force-recreate --no-deps --wait forge
node /tmp/forge_listen_truncation_probe.mjs
docker compose up -d --force-recreate --no-deps --wait forge
cargo fmt --check --all
cargo check -p fdb-gateway
pnpm --filter forge-electric-prototype build
docker compose -f examples/forge-electric-prototype/compose.yml config --quiet
openspec validate pri-c011-forge-watch --strict --no-interactive
openspec validate pri-c012-sdk-pem-network --strict --no-interactive
openspec validate pri-c013-sync-persistence --strict --no-interactive
```

Results:

- Forge format and gateway check: PASS
- Prototype TypeScript/Vite production build: PASS
- Compose configuration validation: PASS
- All three strict OpenSpec validations: PASS
- Focused source review: PASS with no required findings

Vite emitted dependency warnings for PGlite's browser-external NodeFS exports,
use of `eval` inside upstream PGlite bundles, and a bundle over 500 kB. These
warnings did not fail the build or the browser flow.
