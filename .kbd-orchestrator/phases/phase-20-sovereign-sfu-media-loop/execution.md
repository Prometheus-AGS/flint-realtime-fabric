# Execution — phase-20-sovereign-sfu-media-loop

> Stage: Execute · 2026-07-08 · **Backend: OpenSpec** (`kbd-apply detect` ⇒ `openspec`)
> Source: `plan.md` (6 changes). Task walking owned by **`/kbd-apply`**, one task/turn,
> run from the **repo root** (backend detected from cwd).

## Backend selection

**OpenSpec** — project `change_backend`, `openspec/` present, QA gate infra
(`.kbd-orchestrator/bin/qa-gate.sh` + `constraints.md`) in place. Each change:
`/opsx:new` → `proposal.md` + spec delta (`## ADDED Requirements`, **MUST on the line right
after the header**, written **before** the QA gate) + `tasks.md`; walked task-by-task; then
`verify` + `archive` (`openspec archive … --yes` directly — the driver's archive wrapper
swallows the interactive prompt).

> **Hard invariant:** never bare `/opsx:apply`. Drive one task at a time via `/kbd-apply`.

## Dispatch contract (6 changes, in order)

| # | Change ID | QA gate | Skip rationale |
|---|-----------|:------:|----------------|
| c001 | `p20-c001-media-transport-port-adr` | skip | docs-only (ADR-005) |
| c002 | `p20-c002-media-transport-port-trait` | **yes** | — |
| c003 | `p20-c003-str0m-async-session-loop` | **yes** | — |
| c004 | `p20-c004-trickle-ice-wiring` | **yes** | — |
| c005 | `p20-c005-dtls-connected-milestone` | **yes** | — |
| c006 | `p20-c006-g5-reaffirm-and-phase-21-seed` | skip | docs + verification-only close |

## Per-change QA gate (artifact-refiner)

For c002–c005: after the last task reaches DONE, run
`.kbd-orchestrator/bin/qa-gate.sh <change-id>`; ALL PASS → verify + archive; ANY FAIL →
mark BLOCKED, refine, re-run. Standing Rust gates run inside each change first: `cargo fmt
--check`, `cargo clippy --lib --bins -- -D warnings -W clippy::pedantic
-W clippy::unwrap_used`, `--tests`, `cargo test`.

## Standing constraints (CLAUDE.md — every code change)

- No file >500 lines — split into a directory module (watch `session.rs` in c003–c005).
- No `unwrap()`/`expect()` in library crates (`frf-ports`, `frf-media-str0m`); `anyhow`
  only at binary edges.
- `#[non_exhaustive]` public enums; newtype IDs; `tracing` spans across port boundaries.
- **`frf-ports` stays implementation-free** (dependency rule) — c002 is a trait only.
- **`SFU_MODE=sovereign` stays gated off** through the whole phase — it reaches DTLS-
  connected, not media-forwarded. Do not flip the gate.

## Per-turn signals (relayed from the apply driver)

```
Starting change <N> of 6: <change-id>
Starting task <i> of <n>: <title>     ← kbd-apply begin-task
Completed task <i> of <n>: <title>    ← kbd-apply end-task
Completed change <N> of 6: <change-id>
```

## First dispatch

`/kbd-apply p20-c001-media-transport-port-adr` — the ADR that unblocks all G1–G3 engine
work. `/opsx:new p20-c001-media-transport-port-adr` scaffolds the change first.
