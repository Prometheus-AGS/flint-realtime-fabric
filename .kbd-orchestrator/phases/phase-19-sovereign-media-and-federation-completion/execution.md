# Execution — phase-19-sovereign-media-and-federation-completion

> Stage: Execute · 2026-07-07 · **Backend: OpenSpec** (`kbd-apply detect` ⇒ `openspec`)
> Source: `plan.md` (7 changes). Task walking is owned by **`/kbd-apply`**, one task/turn.

## Backend selection

**OpenSpec** — the project's `change_backend`, `openspec/` present, QA gate infra
(`.kbd-orchestrator/bin/qa-gate.sh` + `constraints.md`) in place. Each change is an
OpenSpec change: `/opsx:new <id>` → `proposal.md` + spec delta (`## ADDED Requirements`,
written **before** the QA gate) + `tasks.md`; walked task-by-task via `/kbd-apply`; then
`verify` + `archive` (archive needs `--yes`).

> **Hard invariant:** never run bare `/opsx:apply` / any "do-everything" command. Drive one
> task at a time through `/kbd-apply` so KBD stays the source of truth (hooks, progress.json,
> waypoint). Run the apply driver from the **repo root** (it detects backend from cwd —
> phase-18 c004 silently no-op'd from a subdir).

## Dispatch contract (7 changes, in order)

| # | Change ID | QA gate | Skip rationale (if skip) |
|---|-----------|:------:|--------------------------|
| c001 | `p19-c001-admin-ui-lint-debt` | **yes** | — |
| c002 | `p19-c002-atproto-outbound-gateway-wiring` | **yes** | — |
| c003 | `p19-c003-dart-async-defer-reaffirm` | skip | docs-only (deferral re-affirmation) |
| c004 | `p19-c004-oidc-idp-adr` | skip | docs-only (ADR) |
| c005 | `p19-c005-livekit-inbound-relay-capability` | **yes** | — |
| c006 | `p19-c006-str0m-udp-transport-loop-spike` | **yes** | — |
| c007 | `p19-c007-phase-20-seed-and-docs-close` | skip | docs + verification-only close |

## Per-change QA gate (artifact-refiner)

For c001, c002, c005, c006: after the last task reaches DONE, run
`.kbd-orchestrator/bin/qa-gate.sh <change-id>` (reads `constraints.md`); ALL PASS → verify
+ archive; ANY FAIL → mark BLOCKED, refine, re-run. **Write the spec delta before the
gate** — its P1 runs `openspec validate` (phase-18 c005 lesson). Standing Rust gates
(`cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic
-W clippy::unwrap_used`, `--tests`, `cargo fmt --check`, `cargo test`) run inside each
Rust change before the QA gate.

## Standing constraints (CLAUDE.md — apply to every code change)

- No file >500 lines — split into a directory module (c005 adapter, c006 spike).
- No `unwrap()`/`expect()` in library crates (`frf-media-livekit`, `frf-media-str0m`,
  `frf-bridge-atproto`); `anyhow` only at binary edges (`frf-gateway`).
- `#[non_exhaustive]` on public enums; newtype IDs; `tracing` spans across port boundaries.
- **Security:** ATProto app-password (c002) is a credential — env/secret only, never logged
  or committed. `SFU_MODE=sovereign` stays **gated off** through c006 (spike, not shipped).

## Per-turn signals (relayed from the apply driver)

```
Starting change <N> of 7: <change-id>
Starting task <i> of <n>: <title>     ← from kbd-apply begin-task
Completed task <i> of <n>: <title>    ← from kbd-apply end-task
Completed change <N> of 7: <change-id>
```

## First dispatch

`/kbd-apply p19-c001-admin-ui-lint-debt` — begins the phase with the smallest, self-contained
win (both lint errors already reproduce). `/opsx:new p19-c001-admin-ui-lint-debt` creates
the change scaffold first.
