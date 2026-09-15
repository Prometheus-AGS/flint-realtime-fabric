# pri-c001 baseline

Captured: 2026-09-15
Repository: flint-realtime-fabric
Branch: codex/production-readiness-integration
Base commit: 3043ca53239cc74a7f94a9e12484e2fd17726adb
Dependencies: none

## Affected-source hashes before implementation

- Cargo.toml: d6fbddcf679bc8c4ee8236f708f0627da346f76d677a682bf1b3271d11d40111
- .github/workflows/ci.yml: 1b5710e8132b459e4f9b4447ffd7d293b323bcdb31cbb998b182245b905b6fff
- crates/frf-gateway/src/routes/dev.rs: 04da0bc8313533f701e5acf3e1a01cbbe2c3ef26933c431618bcddb5ece2f307
- openspec/config.yaml: cbf1bbba36adedb9bc05a24abf0688f6e0351c2cec98516ab814556d64c41ffb

The Dagger surface consists of dagger/codegen.ts, package manifests, README and
tsconfig. Workflow files are ci.yml, decode-proof.yml, docs-pages.yml and
publish-candidate.yml.

## Demonstrated starting defects

- Cargo.toml declares rust-version 1.94 while ci.yml labels and installs 1.85.
- dev.rs constructs SignalEnvelope without its required subject field.
- ci.yml executes cargo test --all.
- decode-proof.yml executes a browser/runtime decode proof on a hosted runner.
- dagger/codegen.ts executes Playwright twice and retains a known-broken
  flutter_rust_bridge Dart generation stage despite ADR-003 selecting
  uniffi-bindgen-dart.

## Worktree ownership

The pre-existing edits in crates/frf-app/src/shape/mod.rs,
crates/frf-app/src/shape/tests.rs and crates/frf-gateway/src/routes/shape.rs are
user-owned and excluded from this change. Their entry hashes remain in
entry-state.json. KBD migration/proposal artifacts are phase-owned and separate
from this source change.
