# Baseline — pri-c004-deployment-profiles

Captured: 2026-09-15T09:25:15Z

## Source state

| Repository | Revision | status SHA-256 |
|---|---|---|
| Fabric | `4c2a8326c932ebe0f80799454b0cbbf378e53811` | `a2af0b4ccbcde44c140bcb187a5d64b0b16f8c7043b66962679d0c1b55173ffd` |
| Gate | `de88f40dbfe77bc03203bafc2b64de9815fe27b2` | clean, `e3b0c442…b855` |

Fabric's status includes KBD runtime artifacts, the future OpenSpec proposals,
and the three pre-existing shape files recorded by `entry-state.json`. They are
outside this change unless an implementation edit is explicitly recorded in the
final receipt. Gate is clean at this boundary. Any later drift must be recaptured
before Gate evidence is accepted.

## Affected-path fingerprints

| Path | SHA-256 |
|---|---|
| `.env.example` | `7bd59ab7f094c2e5a0878d69d3aab6eff390c83ce02f496235b89f9473cb789f` |
| `compose.yml` | `a58fb429583493ea86c54d4e41741a4371b367c84190b92f3384e4f9ecda5fc9` |
| `crates/frf-gateway/src/config/mod.rs` | `29af9fe32323bcdd7da1cce11d2db30bd095ebb6a6a61ba71a6b4872b86598b9` |
| `docs/ENVIRONMENT.md` | `0f41fe87728f7e0d15269ef0cd205bc113606df526b069fd92ed900dada0f114` |
| `k8s/overlays/ssr/PROVENANCE.md` | `9ada13e5bd8c04509a26b8778b9f049627f61319e4ffb7d4002187fb64d51a8e` |
| `k8s/overlays/ssr/disruption-budget.yaml` | `62c33ae56075903cd9c7db4abaf0497a366c13a35853c4a53d26b27e172dfcda` |
| `k8s/overlays/ssr/gateway.yaml` | `478962eebc7b3996924b9614b5388356f297209ea7ccee0333930d7356b91f0e` |
| `k8s/overlays/ssr/iggy.yaml` | `be9dc2a9c0c96df765fceb706a69827b2494391f68b04e86aef8ca9e8cf6dafb` |
| `k8s/overlays/ssr/kustomization.yaml` | `842a5e9ae122bee8303f99e155d2e58dfa3d21062456600b47841c1d7f26cb6f` |
| `k8s/overlays/ssr/network-policy.yaml` | `5d461f468f89dd01bb5ef7a162a1c3718656c4beb26fecf1604f2b2a1a8cb5a9` |

## Dependency receipts

The three declared predecessor changes are archived and their source-bound
receipts are present.

| Change | Receipt SHA-256 | Result used here |
|---|---|---|
| `pri-c001-build-policy` | `252cee314685c9ce6d4be2c5d68a56eb79992cc87ecd354b0e61abdb727e574b` | build and local-only test policy passed |
| `pri-c002-local-fixtures` | `4409fe7fa6ca1db3b01091ea8160fb4317a8102b8fd1e5074912177185716622` | authenticated real-Iggy fixture passed |
| `pri-c003-scope-research-contract` | `84583fc30b6b5ef13bd41ace5d02b2447f9133da40bb4ee3cd2ed9c09778c366` | Q2 selected independent `full` and `shape-only` profiles |

## Local tooling

- Docker client/server: `29.4.0`
- Docker Compose: `5.5.1`
- Runtime tests remain local by repository policy.
