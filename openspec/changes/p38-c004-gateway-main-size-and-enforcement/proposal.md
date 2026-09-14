# p38-c004 — Repair the file-size violation and add enforcement

## Summary

`crates/frf-gateway/src/main.rs` is 504 lines against a hard 500-line cap. Constraint **R5**
in `.kbd-orchestrator/constraints.md` marks that **BLOCKING** — it fails the project's own QA
gate. It is currently the only such violation in the workspace.

## Evidence

The file was 487 lines before commit `788637a` and is 504 after: **this session's own commit
caused it**, by extracting `ensure_entities_channel` into the same file rather than a new
module.

Four more files sit within 25 lines of the cap and will cross it next:

| File | Lines |
|---|---|
| `crates/frf-gateway/src/config/mod.rs` | 498 |
| `crates/frf-sdk-rust/src/shape.rs` | 495 |
| `crates/frf-app/src/shape/tests.rs` | 484 |
| `crates/frf-app/src/shape/lease.rs` | 477 |

The cap is stated in CLAUDE.md and in `constraints.md` and enforced by **nothing** — which is
why it was crossed silently rather than caught.

## Why this is independent

It touches `crates/frf-gateway/` only. No dependency on c001's outcome.

## Scope

Move `ensure_entities_channel` (and plausibly the telemetry init) out of `main.rs` into a
`bootstrap` module. Then add an enforcement check so the next overage fails visibly — a
script or `just` target, or a CI **lint** step. CI may lint; per AGENTS.md it may never run
tests, and a line-count check is a lint.

## Non-goals

- Splitting the four near-cap files. They are recorded here as the next to cross, not fixed.

## Files

`crates/frf-gateway/src/main.rs`, a new `crates/frf-gateway/src/bootstrap.rs` (or module
dir), and the enforcement check.
