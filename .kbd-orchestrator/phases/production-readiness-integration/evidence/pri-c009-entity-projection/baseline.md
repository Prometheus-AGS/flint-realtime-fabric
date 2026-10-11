# c009 baseline — durable entity projection

Captured: `2026-09-15T20:31:00Z`

Status: **PASS; IMPLEMENTATION NOT STARTED**

## Source identity

- Fabric revision: `fe5a387da928d4a7414a798814d01459f9163565`
- `crates/frf-gateway/src/main.rs`: `3be852f0c04bba34d7e6ce06c37bd7e73b04d064d6f64c0468a48fed0460987f`
- `crates/frf-app/src/entity.rs`: `a0e806aed521a48952db8203a1b484c7abaf0504d7809e95140bbaf306cb1381`
- `crates/frf-ports/src/entity_store.rs`: `96e7a47abefda49ee64ab449db8fdb8f4e92062ecfd9be05dce5c0812fdf817d`
- `crates/frf-ports/src/lib.rs`: `58d56c52be0e3c2f31f8522981aee6d2e292cba3442d7179c96c5fa7db338f7d`
- `crates/frf-store-surreal/Cargo.toml`: `ea591a8ae96e31011cb119251b6717eafe151ee8f08ff2ed3511fbd336873148`
- `crates/frf-store-surreal/src/lib.rs`: `51f17ab82acacd9241ff757613609538d1a56a4845307a51331ffe4cc51fbce8`
- `crates/frf-store-surreal/src/store.rs`: `4686e4fb310f0028e3ffc8920200e850480e0770c4a52803cc7fc9a0db362dfd`

The listed c009 product paths were clean except for
`crates/frf-ports/src/identity.rs`, which contains pre-existing work outside
c009 and is excluded from ownership. Existing shape and identity changes remain
outside this slice.

## Dependency receipts

| Dependency | Accepted revision | Result |
|---|---|---|
| `pri-c004-deployment-profiles` | `315f8494fafcad9c7f8f42b58b791607f79cd9d9` | Source-bound profile checks and final independent review passed. |
| `pri-c005-authority-lifetime` | `ec142e863944224f75f8111281ea15cf887e4dad` | Local authority lifetime, revocation and cancellation checks passed. |
| `pri-c008-cdc-commit-mapping` | `fe5a387da928d4a7414a798814d01459f9163565` | PostgreSQL/Iggy commit mapping and checkpoint safety passed all ten live scenarios and final review. |

## Baseline gap

`EntityService` is implemented, but gateway composition constructs a fresh
`InMemoryEntityStore`. No CDC or broker consumer populates it, and restart loses
both state and watch continuity. `frf-store-surreal` currently owns the separate
`CrdtStore` port, so c009 will use a dedicated projection adapter rather than
make one adapter crate implement a second port.

## Library qualification input

Current SurrealDB Rust SDK documentation and locally installed `3.1.5` source
confirm the consuming transaction handle (`begin`, transaction-scoped query,
`commit`/`cancel`) and bound query APIs. The local image selected for acceptance
is `surrealdb/surrealdb@sha256:408c6930d730adcf65d4bf91f385381e75d9db32663a33ee1b026c4d43914732`
(image id `sha256:3ad3fe6160ada702242506d7134744e065052e380b716a00cd98893b61d05bc8`).
