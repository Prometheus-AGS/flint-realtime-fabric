# c006 watch-contract baseline

Captured: 2026-09-15

## Dependency and governing sources

- Fabric base revision: `ec142e863944224f75f8111281ea15cf887e4dad`
- Required c003 receipt SHA-256:
  `84583fc30b6b5ef13bd41ace5d02b2447f9133da40bb4ee3cd2ed9c09778c366`
- Accepted c003 release contract SHA-256:
  `29752a3109a4602a1edcf14e72be70b6f5b43270ba7ea165b3de9ab148c857bd`
- RFC-FRF-002 SHA-256:
  `67d1fa6ce169f75fa3fce2c7486ec4ea0a791f549eb595a461baf01997be3ed1`
- Prometheus base rules SHA-256:
  `c7a154f33623311fb0f3d93193dd8e47578652c2c632336d186870a3690df849`

The completed c003 contract supplies the required Q4–Q6 decisions: durable
at-least-once delivery, stable event deduplication identity, separately modeled
source/broker/client positions, explicit resnapshot outside retained history,
canonical single/composite/non-UUID keys, authorized key-only deletes, a
24-hour minimum retention target, bounded slow-client handling, and the accepted
scale/recovery objectives.

## Fabric scope

The affected paths `proto/flint/`, `crates/frf-domain/src/`,
`crates/frf-ports/src/` and `docs/decisions/` had no staged or unstaged changes
at capture. Their scoped status and binary-diff SHA-256 values are both the
empty digest `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Base tree identities:

- `proto/flint`: `c182d9dc4acef76d1ad618b156fbdceefac29785`
- `crates/frf-domain/src`: `626b053610c1e7d9dd0bab75cad1ca96e28d2056`
- `crates/frf-ports/src`: `4f9699448486a16cb33228e37ecdf8317d4043e2`
- `docs/decisions`: `0fd8683c8ca160077c9367571bef65b047392da9`

Current v1 file hashes, which c006 must preserve byte-for-byte:

| File | SHA-256 |
|---|---|
| `agent.proto` | `d5908e755d653c05665c632a097a293ead828429362adaf60af78200cf8e4220` |
| `authz.proto` | `258f2b3676ce39b435492b63f88813dba4090a34e660ebaf116f9f288d4efdce` |
| `entity.proto` | `f90e93b6a806039603a76bff9e47d359838cda97f5fb0b418009c16a22fbbcda` |
| `envelope.proto` | `dac772037b2895fef78fa980971d256570e6d9e21502bde2ede926b99c73384c` |
| `signal.proto` | `39ffc96405e9c972fb49d90d01de47508fa72ed24ab59c08556c74bd30daaeba` |
| `sync.proto` | `47925c6e1a3466abd8aed2eed32af82f67eb85daf5eea54ba6e0e49e1bbf8662` |

The historical `proto-v1` tag resolves to
`5d36c8f17a3f9ed060db5cf4866a380e910b88b0` but is not byte-identical to the
current v1 tree: later commits changed package-generation options and changed
`RunAgent` to bidirectional streaming. c006 does not rewrite that history or
claim tag equality. Its compatibility gate freezes the current accepted v1
bytes above and compares new v2 additions without modifying them.

## Consumer baselines

Forge is clean at `dc313be3a044c65b05d845a9c34350bf5ca3ca3e`.
Its current FRF adapter explicitly fails `WatchEntityType` closed and the local
LISTEN adapter remains the opt-in workaround. Relevant source hashes:

- `crates/fdb-realtime/src/lib.rs`: `f549792d6a78b45c80a9ae433111f7dd46aa7565dd2ebb335a528c8a621c6713`
- `crates/fdb-realtime/src/listen/mod.rs`: `ede44827dfe14df80865c3363eb7eee7b196bc009aa990746752d721dc464d74`
- `crates/fdb-domain/src/lib.rs`: `189469edfec19667ac700b5014ce65a140bb63886c9adfda66c3a4f6b5df9109`
- `crates/fdb-gateway/src/subscriptions.rs`: `3d37c2c1a06a2d8c5ad3cf5740ed06774a58f0a3049d826c9426caa818c64fd1`
- `crates/fdb-gateway/src/realtime_source.rs`: `8d0155d75e97ca4207ad01ad1cc942be4733a95b032ce0f9a73ea61690f89b64`

PEM is clean at `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565`.
Its adapter contract consumes an async entity-change stream and requires stable
entity type, key, operation and checkpoint mapping. Relevant source hashes:

- `packages/entity-graph-core/src/adapters/flint.ts`: `f45c11bc928c245ae47cc6be5c139f9870dff9b1f09b0f27540f729f2bc9f1f2`
- `packages/entity-graph-core/src/adapters/types.ts`: `5398200e59d457dabb49dd3cec31779b9760a06c1c812446ec8aa80117657010`
- `packages/entity-graph-core/src/adapters/flint-live.fixture.ts`: `45944fc98d63472017f75d678e00dac9288989e56461d71f54b3c8bc852beed0`
- `packages/entity-graph-core/src/adapters/__fixtures__/flint-contract.ts`: `d705775d106db50eadcd290abc2d6492a139825d25da30c38b812045f359e3e3`

No c006 consumer code is generated or modified before this baseline and the
consumer-acceptance record are complete.
