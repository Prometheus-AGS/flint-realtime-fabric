# c006 source-bound acceptance receipt

Status: **PASS**

Captured: `2026-09-15T14:58:03Z`

## Source identity

- Fabric base revision: `ec142e863944224f75f8111281ea15cf887e4dad`
- Product diff SHA-256: `42fe670d35acefd8329ad79cfd1f4d650f877682bfd019ec363d0f80c849960a`
- Scoped status SHA-256 before this receipt:
  `6ca99c5f59a4c509d43067c72ad38ef04d5203cd62bffe4f6f9bec0864312eb7`
- Forge consumer baseline: clean
  `dc313be3a044c65b05d845a9c34350bf5ca3ca3e`
- PEM consumer baseline: clean
  `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565`
- c003 release contract SHA-256:
  `29752a3109a4602a1edcf14e72be70b6f5b43270ba7ea165b3de9ab148c857bd`

## Acceptance results

- Consumer acceptance: recorded before any c006 proto edit/code-generation run,
  with Forge/PEM mapping and error behavior fixed at exact revisions.
- Contract fixture: PASS — 6 frozen current-v1 files, 3 required key shapes,
  4 resume cases, committed transaction order, stable IDs, snapshot barrier,
  projection/delete, unauthorized history, bounded lag and cancellation.
- `buf lint proto`: PASS.
- `buf breaking proto --against '.git#ref=HEAD,subdir=proto'`: PASS under FILE
  compatibility. Context7 `/bufbuild/buf` confirmed the Git input syntax.
- `buf build proto -o /tmp/frf-c006-current.binpb`: PASS; descriptor SHA-256
  `643f268857f402403ce286cd231337b8cb97dd03659726cea5098d77abdba219`.
- `cargo test --offline -p frf-proto -p frf-domain -p frf-ports --all-targets --locked`:
  PASS, 15 tests total. Five generated tests include simultaneous v1/v2 use;
  ten unchanged domain round-trip tests also pass.
- Strict all-target Clippy for those three crates: PASS with warnings denied.
- `cargo check --offline -p frf-gateway -p frf-sdk-rust --locked`: PASS.
- `cargo check --workspace`: PASS across the repository.
- `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic`:
  PASS.
- `cargo clippy --workspace --lib --bins --features frf-gateway/dev-endpoints -- -D warnings -W clippy::pedantic`:
  PASS.
- `cargo clippy --workspace --tests -- -D warnings -W clippy::pedantic`:
  PASS.
- Rust format, Node syntax, deterministic fixture, artifact manifest/JSON,
  touched-file 500-line limit and `git diff --check`: PASS.
- `openspec validate pri-c006-watch-contract --strict --no-interactive`: PASS.
- `git diff --exit-code HEAD -- proto/flint/v1`: PASS; all six baseline hashes
  also match the deterministic checker.
- All tests ran locally. No CI test run was used.

## Review disposition

Initial independent review: **BLOCK**, 1 critical, 0 warnings, 0 suggestions.

- The finding observed that task 5 was still open because independent review
  was one of its prerequisites.
- After the review, every current local gate was rerun, this source-bound
  receipt was refreshed, and the required finding was resolved by completing
  task 5 only after those prerequisites passed.
- A fresh independent resolution review is required before archive.

First resolution review: **BLOCK**, 1 critical, 0 warnings, 0 suggestions.

- The judge required the workspace-wide R1/R2 commands in addition to the
  already-passing affected-package checks.
- The workspace compile, primary library/binary Clippy, `dev-endpoints` feature
  Clippy, and test-target Clippy all pass locally with warnings denied. The
  exact commands are recorded above.
- A second fresh resolution review is required before archive.

Second resolution review: **PASS**, 0 critical, 0 warnings, 0 suggestions.
The strict sycophancy screen also passed. All required findings are resolved.

## Artifact hashes

- v2 proto: `02ea2bf4db1a86a21e2cae25c4f2e6a2cd3c566fb158d3391f637f518d0df528`
- fixture vectors: `b9bdf9e37ff9ca18b4a610ce338f5443aa6df8336b787fe782a8f2abfd50ed29`
- ADR-010: `bfe7555ddee03ca312cd4fded85a8cb7c0049635c3f1e3e37963d8e039de9cf0`
- generated-contract tests: `019ced5515effc934e7a792cb1d90d1575ad0cb59fe531683a090312c9f4a834`
- deterministic checker: `871eb326801a052931c7bc4ab73279733cad92950fc5ccdff260f771017746f2`

## Compatibility and limits

The new package/module is additive on the wire and requires at least the next
minor pre-1.0 `frf-proto` release at c023. `frf-domain` and `frf-ports` are
unchanged in c006; later public watch types/ports receive the same minor-version
treatment. ADR-010 reserves a dedicated watch-source port and forbids adding a
second port implementation to existing LogBroker/EntityStore adapter types.

c006 freezes the interface and recovery semantics. It does not claim that the
CDC mapper, broker replay, projection, gateway service, Forge adapter or packed
SDK transport exists; those remain c007–c012 gates.
