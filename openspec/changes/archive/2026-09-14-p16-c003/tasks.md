# Tasks — p16-c003

- [x] Add unwrap_used = deny to workspace [lints.clippy]
- [x] Fix frf-crdt unwrap/expect sites (~36)
- [x] Fix frf-store-redb unwrap/expect sites (~16)
- [x] Fix remaining library-crate sites
- [x] #[allow] + justify any truly-unreachable cases
- [x] Update ci.yml clippy invocation to include unwrap_used
- [x] Update dagger clippy stage to match
- [x] cargo clippy --workspace passes with -D warnings

## Notes

### The ~50 estimate was almost entirely test code

The audit's "~50 unwraps (frf-crdt 36, frf-store-redb 16)" counted `#[cfg(test)]`
and `tests/` code. After enabling `clippy::unwrap_used`/`expect_used` (a proof-probe
confirmed the lint fires), the workspace had **zero** unwrap/expect violations in
frf-crdt or frf-store-redb library code. Tasks 2 and 3 as literally worded were no-ops;
the real production-code violations the stricter gate surfaced were different and fewer.

### What was actually enforced and fixed

Centralized lint policy in `[workspace.lints.clippy]` (was per-crate + inconsistent —
also closes assessment finding #45): `pedantic = warn`, `unwrap_used = deny`,
`expect_used = deny`. Every one of the 22 crates now uses `[lints] workspace = true`.

`clippy.toml` scopes the restriction lints to production code:
`allow-unwrap-in-tests = true` / `allow-expect-in-tests = true` (covers `#[cfg(test)]`).
Integration-test and bench crates (separate compilation units where helpers sit at
module scope, outside a `#[test]` fn) carry a crate-level
`#![allow(clippy::unwrap_used, clippy::expect_used)]`.

Real production-code fixes (all pre-existing, surfaced by consistent enforcement):
- `frf-proto/build.rs` — `expect()` → `ok_or(...)?`
- `frf-policy-cedar/src/lib.rs` — 2× unnecessary `to_string()` on `&str`
- `frf-librefang/src/bus.rs` — missing `# Errors` doc on `start()`
- `frf-bridge-matrix`, `frf-gateway/{config,routes/dev}.rs` — `doc_markdown` backticks
- `frf-gateway/src/config.rs` `test_default()` — `expect()` on a parsed addr →
  infallible `SocketAddr::from(([127,0,0,1], 0))`
- `frf-gateway/src/config.rs` `dev_no_auth()` — `#[must_use]` + `map().unwrap_or()`
  → `is_ok_and()`
- `frf-gateway/src/main.rs` — OTLP exporter `expect()` → `context(...)?`
  (init_telemetry now returns Result); fixture UUID `parse().expect()` →
  infallible `Uuid::from_u128(1)`; `_tracer_provider` underscore binding renamed
- `frf-gateway/src/routes/{publish,subscribe}.rs` — `manual_let_else` rewrites
- Two pre-existing test defects fixed so the `--tests` pedantic pass is green:
  `frf-media-str0m` (undeclared `chrono` dev-dep; duplicate `StreamExt` import;
  an unconsumed subscription stream in `remove_session_drops_channel`) and
  `frf-gateway/tests/signal_mux.rs` (doc backticks).

### CI/dagger gate wiring (operator decision: gate on --lib --bins)

The restriction gate (`unwrap_used`/`expect_used`) runs on PRODUCTION code only —
`cargo clippy --workspace --lib --bins`, in BOTH the default and
`--features frf-gateway/dev-endpoints` configs (so the dev-endpoints auth-bypass path
is also linted). A separate `--tests` pass keeps `pedantic` but tolerates unwrap in
tests. Wired identically in `.github/workflows/ci.yml` and `dagger/codegen.ts`.

### Verification

- `cargo clippy --workspace --lib --bins` (default) → exit 0
- `cargo clippy --workspace --lib --bins --features frf-gateway/dev-endpoints` → exit 0
- `cargo clippy --workspace --tests -- -D warnings -W clippy::pedantic` → exit 0
- `cargo test --workspace --lib` → all pass
- `cargo fmt --check --all` → clean
