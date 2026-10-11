# c010 baseline — authorized resumable entity-type watch

Captured: `2026-09-16T13:37:57Z`

Status: **PASS; implementation not started**

## Source identity and scoped dirt

- Fabric revision: `2ccabf50141ded7edc89f418c2719441221d2219`
- `crates/frf-app/src` tree: `e22bf207628e001b0b5c9ee93add8e495e55b60e`
- `crates/frf-gateway/src` tree: `d8cea0fdfb71692ea7e183052ab21b4efb1e729c`
- `crates/frf-proto` tree: `dd88b6be222dc75840666e858396101ceedb7d21`
- `proto/flint/v2` tree: `d096379a224f9fcd2a4e35eb2508b2de3c7d3fb3`
- v1 entity schema SHA-256:
  `f90e93b6a806039603a76bff9e47d359838cda97f5fb0b418009c16a22fbbcda`
- v2 build script SHA-256:
  `a87fc5cb5cf2f4cba630cec9a7195eba2b86c852d34d4b53f4b202e164557757`

The v2 proto and `frf-proto` paths were clean. Pre-existing uncommitted work is
confined to the app shape modules/tests, the gateway shape route/test, and
identity claim/port files. Those paths are outside c010 ownership and will not
be staged or rewritten. The committed c009 entity projection is the clean base
for shared entity/gateway paths.

## Dependency receipts

| Dependency | Accepted revision | Receipt SHA-256 | Result |
|---|---|---|---|
| `pri-c005-authority-lifetime` | `ec142e8` | `618b43a8dcf46004028df24687f8ea4df3d5decab43dbe074d8905afe71db45e` | PASS |
| `pri-c006-watch-contract` | `5aaad52` | `50fe1da97f0167ecfef66b06a731a5b3c7d73882011dbcf0c708657ddd07e9b9` | PASS |
| `pri-c007-broker-replay` | `b60e15a` | `cdfccacd63fad4c714b825206564d973e6bdbb3c0cdc7f30d0a38afc08155484` | PASS |
| `pri-c008-cdc-commit-mapping` | `fe5a387` | `2a29dae7251c2e30081a02296591e823848baccd3fa039fc2ac83cba6644b24a` | PASS |
| `pri-c009-entity-projection` | `2ccabf5` | `0929e2975e3dfda5825f26d465e44c16c05cdc3a6be80e265f25cd4a68ec67af` | PASS |

## Frozen implementation boundary

The accepted c006 contract already supplies additive `flint.v2`, generated
tonic client/server stubs, canonical typed keys and records, opaque checkpoint
messages, terminal resnapshot/lag controls, and the exact Forge mapping. c010
must implement that existing schema without editing `proto/flint/v1`.

The c008 committed `CdcMutation` payload is the authoritative live record and
c007 supplies explicit broker offsets, retained replay, acknowledgements and
expired-position failure. c009 supplies durable current entity state and
projector readiness. A dedicated type-watch source port/adapter will translate
the durable broker payload; existing `LogBroker` and `EntityStore` adapter types
will continue to implement one port each.

## Local-only verification rule

All runtime proof will use owned local fixtures. No CI test invocation or CI
result can satisfy a c010 acceptance gate.
