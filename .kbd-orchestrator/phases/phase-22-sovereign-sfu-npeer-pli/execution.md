# Execution — phase-22-sovereign-sfu-npeer-pli

> Stage: Execute · 2026-07-08 · **Backend: OpenSpec** (`kbd-apply detect` ⇒ `openspec`)
> Source: `plan.md` (4 changes). Task walking owned by **`/kbd-apply`**, one task/turn, from
> the **repo root**.

## Backend selection

**OpenSpec** — project `change_backend`, `openspec/` present, QA-gate infra in place. Each
change: `/opsx:new` → `proposal.md` + spec delta (`## ADDED Requirements`, **MUST on the line
right after the header**, before the QA gate) + `tasks.md`; walked task-by-task; then `verify`
+ `archive` (`openspec archive … --yes` directly).

> **Hard invariant:** never bare `/opsx:apply`. Drive one task at a time via `/kbd-apply`.

## Dispatch contract (4 changes, in order)

| # | Change ID | QA gate | Skip rationale |
|---|-----------|:------:|----------------|
| c001 | `p22-c001-npeer-fanout-proof` | **yes** | — |
| c002 | `p22-c002-pli-keyframe-forwarding` | **yes** | — |
| c003 | `p22-c003-gateway-sovereign-composition` | **yes** | — |
| c004 | `p22-c004-carry-e2e-and-close` | skip | docs + verification-only close |

## Per-change QA gate — READ THE VERDICT (phase-21 c004 lesson)

For c001–c003: after the last task reaches DONE, run `.kbd-orchestrator/bin/qa-gate.sh
<change-id>` and **read its verdict line** — `→ PASS` before verify+archive, `→ BLOCKED` means
refine + re-run. **Do not archive on the `verify` step alone** (phase-21 c004 archived on a
clippy BLOCK because the verdict wasn't read). The gate runs `cargo clippy --workspace` from a
clean state — authoritative over a cached per-crate clippy.

Run the heavy gates in the **background** (cold rebuild + str0m/gateway link budget) so tool
timeouts don't interrupt; wait for the completion notification, then read the verdict.

## Standing constraints (CLAUDE.md — every code change)

- No file >500 lines — split `room.rs`/`driver.rs`/`signal_service.rs` if c002/c003 grow them.
- No `unwrap()`/`expect()` in library crates; `anyhow` only at binary edges (gateway).
- `#[non_exhaustive]` public enums; newtype IDs; `tracing` spans across port boundaries.
- **`SFU_MODE=sovereign` stays gated off** — c003 composes the media plane but the sovereign
  branch must NOT advertise live media; the gate flip is carried (c004). Do not flip it.
- **Security:** the media plane rides the already-JWT-authed signal channel; tenant isolation +
  per-event Keto RLS on the media path are a follow-on boundary (c004 re-affirms).

## Per-turn signals (relayed from the apply driver)

```
Starting change <N> of 4: <change-id>
Starting task <i> of <n>: <title>     ← kbd-apply begin-task
Completed task <i> of <n>: <title>    ← kbd-apply end-task
Completed change <N> of 4: <change-id>
```

## First dispatch

`/kbd-apply p22-c001-npeer-fanout-proof` — the cheap N-peer delivery proof on the existing
`RoomRouter` fan-out. `/opsx:new` scaffolds it first.
