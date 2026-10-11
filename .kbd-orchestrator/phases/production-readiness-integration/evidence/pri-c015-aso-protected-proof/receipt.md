# Receipt — pri-c015-aso-protected-proof

Status: **review candidate**

## Qualified capability

The selected browser PGlite `shape-only` topology passed current-source
protected replication from Gate and FRF shapes through transactional SQL
materialization into the PEM graph. The candidate establishes one authoritative
clinical shape writer, atomic row/checkpoint recovery, authorized graph
recovery, practice and logout fencing, interrupted-body recovery, projection
and local-only-field egress, handle expiry/refetch, stream revocation and
client/backend isolation.

## Source identity

| Repository | HEAD | Selected-source aggregate | Files |
|---|---|---|---:|
| ASO | `d95242542060d1e19a6efe9143f57c015d6162f4` | `8c8c60eb49bb56e37aab78c0e2f09a3b17470a47d6381c039f5c00866f501ecd` | 97 |
| FABRIC | `ae26f45d8afa037677004cd118b0571b8b552131` | `60e5ec0775a0b0e96911fa9868e97d6b2e9ae1028288ae5f9303aa2f73976f43` | 384 |
| GATE | `77914a2ad2e3cd57f12fd9131fa8cdddf625f68e` | `95d7b46655809d6aeade146365e44a26c4192144c7658f2d9b07f72b23974315` | 79 |

The final source manifest matches the authoritative campaign for all three
repositories and records every selected file, dirty-state hashes, contract
files, runtime image IDs and OCI source labels.

## Verification

- Authoritative protected campaign: passed.
- ASO focused tests: 99 passed.
- Fabric current revocation suite: 4 passed.
- Retained Fabric revocation and lease-timing suite: 10 passed, maximum 1,500 ms
  against the accepted 5,000 ms bound.
- Mounted durability and ownership checks: 13 passed.
- Mounted Chrome lifecycle checks: 9 passed across two practices.
- Browser interruption and recovery checks: 10 passed.
- Protected composition checks: 10 passed.
- Network topology checks: 9 passed.
- ASO Python compilation, typecheck, lint, production build and architecture
  audit: passed.
- Fabric `cargo check -p frf-app -p frf-gateway`: passed.
- Gate replica-grant tests: passed.
- Strict OpenSpec validation: passed.

The ASO build retained upstream PGlite/Rollup warnings about browser-external
Node filesystem symbols, dependency `eval`, and the existing large main chunk.

The browser run's c014-owned RSS metric exceeded 512 MiB and is disclosed in
`acceptance-b.md` and `receipt.json`; it does not alter c015's passed functional
acceptance.

## Primary evidence

- `receipt.json`: `8b8675076ff34177668f2003afe89a7f825934b94ac3f5b5e5d93fbd2bcdd476`
- `source-manifest.json`: `540d1cf82b10d4dfc7360b2f0f19f046343d7a6276f8bcec96d7ab98df2428e6`
- `quality-checks.json`: `5374c3d61a452671ad84594a808818c9a63cb9fb4845613391c97b16f4e0d786`
- `campaign.json`: `6abf86af9336ece7e2fd32a6229f85d039e9b43b4cc20ff0415aa00cb00bea51`
- `acceptance-a.md`: `a1a42d5f6cd6484c2376e774b7d0f27d6fe639b7ec7d72848e7cb679ae64e70e`
- `acceptance-b.md`: `abc9b7aa865472e5ea1552cc685526b6824a3642a67ec88f8065e3dcf4a733ff`

`receipt.json` inventories 15 primary artifacts by size and
SHA-256. Failed and diagnostic attempts remain explicit historical evidence and
are excluded from the passing claim.
