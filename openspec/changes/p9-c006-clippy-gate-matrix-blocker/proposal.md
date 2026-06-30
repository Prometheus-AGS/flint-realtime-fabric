# p9-c006 — Clippy CI Gate + Matrix Blocker Documentation

## Summary

Enforce `clippy::pedantic` in the Dagger CI rust-check stage; add `BLOCKED_ON_TUWUNEL`
doc comment to `ReqwestMatrixClient`.

## Files to Modify

- `dagger/codegen.ts` — add `-W clippy::pedantic` to Clippy invocation in Stage 1
- `crates/frf-bridge-matrix/src/client.rs` — add blocker doc comment to `ReqwestMatrixClient`

## Exit Criteria

- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
- `dagger/codegen.ts` typechecks
- `ReqwestMatrixClient` has `BLOCKED_ON_TUWUNEL` doc comment
