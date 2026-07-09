# Execution — phase-21-sovereign-rtp-forwarding

> Stage: Execute · 2026-07-08 · **Backend: OpenSpec** (`kbd-apply detect` ⇒ `openspec`)
> Source: `plan.md` (5 changes). Task walking owned by **`/kbd-apply`**, one task/turn,
> from the **repo root**.

## Backend selection

**OpenSpec** — project `change_backend`, `openspec/` present, QA-gate infra in place. Each
change: `/opsx:new` → `proposal.md` + spec delta (`## ADDED Requirements`, **MUST on the
line right after the header**, written **before** the QA gate) + `tasks.md`; walked
task-by-task; then `verify` + `archive` (`openspec archive … --yes` directly — the driver's
archive wrapper swallows the interactive prompt).

> **Hard invariant:** never bare `/opsx:apply`. Drive one task at a time via `/kbd-apply`.

## Dispatch contract (5 changes, in order)

| # | Change ID | QA gate | Skip rationale |
|---|-----------|:------:|----------------|
| c001 | `p21-c001-split-session-driver` | **yes** | — |
| c002 | `p21-c002-offerer-role-and-connected-proof` | **yes** | — |
| c003 | `p21-c003-rtp-fanout-adr` | skip | docs-only (ADR-006) |
| c004 | `p21-c004-rtp-forwarding-1to1` | **yes** | — |
| c005 | `p21-c005-sovereign-gate-and-phase-22-seed` | skip | docs + verification-only close |

## Per-change QA gate (artifact-refiner)

For c001, c002, c004: after the last task reaches DONE, run
`.kbd-orchestrator/bin/qa-gate.sh <change-id>`; ALL PASS → verify + archive; ANY FAIL →
mark BLOCKED, refine, re-run. Standing Rust gates run inside each change first: `cargo fmt
--check`, `cargo clippy --lib --bins -- -D warnings -W clippy::pedantic
-W clippy::unwrap_used`, `--tests`, `cargo test`.

## Standing constraints (CLAUDE.md — every code change)

- **No file >500 lines** — c001 exists *because* `session.rs` hit 498; keep `driver.rs`,
  `session.rs`, and any new `room.rs` each ≤500 (split further if c004's registry grows them).
- No `unwrap()`/`expect()` in library crates; `anyhow` only at binary edges.
- `#[non_exhaustive]` public enums; newtype IDs; `tracing` spans across port boundaries.
- **`frf-ports` stays implementation-free**; str0m implements two distinct ports.
- **`SFU_MODE=sovereign` stays gated off** through c001–c004 and is flipped in c005 **only
  if** 1-to-1 media flows end-to-end — else re-affirmed gated. Do not flip it early.

## Per-turn signals (relayed from the apply driver)

```
Starting change <N> of 5: <change-id>
Starting task <i> of <n>: <title>     ← kbd-apply begin-task
Completed task <i> of <n>: <title>    ← kbd-apply end-task
Completed change <N> of 5: <change-id>
```

## First dispatch

`/kbd-apply p21-c001-split-session-driver` — the behavior-preserving split that clears the
498/500 file-size blocker before any engine code. `/opsx:new` scaffolds it first.
