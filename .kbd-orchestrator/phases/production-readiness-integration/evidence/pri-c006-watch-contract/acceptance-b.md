# c006 acceptance B — v1 preservation and v2 compatibility

Captured: `2026-09-15T14:37:56Z`

Status: **PASS**

## Frozen v1 bytes

`git diff --exit-code HEAD -- proto/flint/v1` exited 0. The deterministic
contract checker independently recomputed all six SHA-256 values and matched the
task-1 baseline:

| File | SHA-256 |
|---|---|
| `agent.proto` | `d5908e755d653c05665c632a097a293ead828429362adaf60af78200cf8e4220` |
| `authz.proto` | `258f2b3676ce39b435492b63f88813dba4090a34e660ebaf116f9f288d4efdce` |
| `entity.proto` | `f90e93b6a806039603a76bff9e47d359838cda97f5fb0b418009c16a22fbbcda` |
| `envelope.proto` | `dac772037b2895fef78fa980971d256570e6d9e21502bde2ede926b99c73384c` |
| `signal.proto` | `39ffc96405e9c972fb49d90d01de47508fa72ed24ab59c08556c74bd30daaeba` |
| `sync.proto` | `47925c6e1a3466abd8aed2eed32af82f67eb85daf5eea54ba6e0e49e1bbf8662` |

The historical `proto-v1` tag discrepancy is documented in `baseline.md`; this
change does not conceal it or claim that current v1 equals the old tag.

## Generated old/new compatibility

- `buf lint proto`: PASS under the repository STANDARD policy.
- `buf breaking proto --against '.git#ref=HEAD,subdir=proto'`: PASS under FILE
  compatibility. Context7 `/bufbuild/buf` documentation confirmed local Git
  comparison syntax and support for `ref`/`subdir` input options.
- `buf build proto -o /tmp/frf-c006-current.binpb`: PASS; descriptor image
  SHA-256 `643f268857f402403ce286cd231337b8cb97dd03659726cea5098d77abdba219`.
- `cargo test --offline -p frf-proto --all-targets --locked`: PASS, 5 tests.
  One test instantiates and encodes both the unchanged
  `frf_proto::fv1::WatchEntityRequest` and new
  `frf_proto::fv2::WatchEntityTypeRequest`; four tests cover v2 generated
  messages and oneofs.
- `cargo clippy --offline -p frf-proto --all-targets --locked -- -D warnings`:
  PASS.
- `cargo check --offline -p frf-gateway -p frf-sdk-rust --locked`: PASS. The
  existing gateway and hand-written Rust SDK compile with both generated
  namespaces present.

## Compatibility and ownership decision

The new `flint.v2` package and `frf_proto::fv2` module are wire-additive and do
not change a v1 field, service, method or Rust module. Because `frf-proto` is
pre-1.0, the public module addition requires at least the next minor release at
c023. Generated Go/TypeScript/C# artifacts and their own package-version effects
remain c012 work; c006 does not hand-edit generated SDK output.

c006 makes no change to `frf-domain` or `frf-ports`, so those crates have no
immediate semver delta. Later typed domain and watch-port additions are public
pre-1.0 additions and also require at least a minor release. ADR-010 reserves a
dedicated watch-source port for its future adapter. Existing `LogBroker` and
`EntityStore` adapter types do not gain a second port implementation; gateway
composition owns v2 service wiring.

## Current artifact hashes

- v2 proto: `02ea2bf4db1a86a21e2cae25c4f2e6a2cd3c566fb158d3391f637f518d0df528`
- fixture vectors: `b9bdf9e37ff9ca18b4a610ce338f5443aa6df8336b787fe782a8f2abfd50ed29`
- ADR-010: `bfe7555ddee03ca312cd4fded85a8cb7c0049635c3f1e3e37963d8e039de9cf0`
- `frf-proto` build script: `a87fc5cb5cf2f4cba630cec9a7195eba2b86c852d34d4b53f4b202e164557757`
- `frf-proto` module exports: `54f88c349b13b521e9ad6582fb27b47fc43bd742c6a80b8f3bbc0fcdbbd152ea`
- generated-contract tests: `019ced5515effc934e7a792cb1d90d1575ad0cb59fe531683a090312c9f4a834`
- deterministic checker: `871eb326801a052931c7bc4ab73279733cad92950fc5ccdff260f771017746f2`
