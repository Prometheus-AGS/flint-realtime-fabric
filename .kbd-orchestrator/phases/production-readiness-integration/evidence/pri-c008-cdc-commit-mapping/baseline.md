# c008 baseline — committed typed CDC mapping

Captured: `2026-09-15T17:45:00Z`

Status: **PASS; IMPLEMENTATION NOT STARTED**

## Source baseline

- Fabric revision: `b60e15a31cfb54058f56e96b76fa4b48c5080172`
- The c008 product paths were clean before implementation.
- `consumer.rs`: `25e3306ebfb1c551605d7f08a942d88964cc7f3f839fbde25740f0c2a7e471fa`
- `decode.rs`: `b2085bd688bc2f79eeb3b964707d12549e6ec1761d299d8949e2fcb45c9038d9`
- `config.rs`: `21b1adfb7978235d6fcea2679da19d75dab829080a9f75bf9242365d85f9fbce`
- `consumer_smoke.rs`: `7507840c2130c8f076e0629207dbe07f3c054de9551a3435c6463f4da68af380`
- `Cargo.lock`: `d23cb3f86cbfe06e53de83865cde9e3a2c2e856b4615113f27c3901f54419b5c`

Unrelated workspace modifications were present outside these paths and remain
outside c008 ownership.

## Dependency receipts

| Dependency | Accepted revision | Evidence |
|---|---|---|
| `pri-c002-local-fixtures` | `52ab9b3874abbe35bbca73720d85578f3db70943` | Isolated local fixture and mandatory failure semantics passed. |
| `pri-c006-watch-contract` | `5aaad523ad6803bd07992a6ca6e6d75238d968c9` | Versioned source/key/value/transaction contract passed independent review. |
| `pri-c007-broker-replay` | `b60e15a31cfb54058f56e96b76fa4b48c5080172` | Durable authoritative offsets, manual acknowledgement, restart replay and retention failure passed local acceptance and independent review. |

## Pinned parser evidence

`Cargo.lock` resolves `pg_walstream 0.6.3`. Context7 was queried twice under
both `pg_walstream` and `pg-walstream`; it returned no matching library. The
exact installed crate source is therefore the API authority for this slice:

- `types.rs`: `fe12125ae1dd62c5a908ca20e030e5a7de6357e6087b0cb3ec2339bd2d37d765`
- `protocol.rs`: `bfa58f477b5eaa6d9d4e2378117e04972850e2756c652572535811abf897f61c`
- `stream.rs`: `4c1981bc92c30f0bd90b472fdb8b5df9c3eae31c4cbf1eb91292cf41ce77c892`

The API exposes BEGIN/COMMIT and row events, relation column metadata including
key flags and type OIDs, replica identity, and separate applied-LSN feedback.
Its tuple conversion omits unchanged TOAST fields. These facts make explicit
transaction buffering, enrollment validation, schema-drift checks and poison
transaction handling mandatory in c008.

## Demonstrated starting defects

- A process-local envelope counter is used as the source offset.
- Row events publish before COMMIT and applied LSN advances after each row.
- Decode failures are warned and skipped beneath later acknowledged LSNs.
- Every relation is fabricated with primary key index zero.
- All key values are required to parse as UUIDs.
- Row values are flattened to nullable strings and schema is omitted from the
  entity type.
- Relation/schema events, replica identity, transaction boundaries and
  unchanged TOAST are ignored.
- The planned real integration test does not exist yet.

No runtime or certification claim is inherited from earlier CDC smoke tests.
