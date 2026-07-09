# Execution — phase-31-sovereign-sfu-decode-prebuilt-image-and-flip

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`).

## Backend selection

**`openspec`** — `openspec/` present; the phase updates the `sfu-mode-consistency` spec per gate
decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of
its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated, verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p31-c001-prebuilt-image-runner` | runner presence-check + fail-fast + `PREBUILD_GATEWAY` escape hatch; header/docs | full (script + docs) | verify → archive |
| 2 | `p31-c002-decode-run-and-flip` | decode re-run against pre-built image on restarted daemon; DECODE-RESULT; conditional flip-or-reaffirm; SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived. c002 last (gate decision).

## Operational prerequisite for c002 (manual, operator runs via `!`)

The daemon is DOWN and the image must exist before the decode run:

```
! colima start --memory 8
! docker compose -f compose.yml -f compose.sovereign.yml build gateway
```

c001 makes this a fail-fast requirement (no silent in-run rebuild).

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / S1
no-secret / P1 openspec validate. ANY FAIL → mark BLOCKED + refine; do not archive.

## Invariants (carried 16→30)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (phase-29 lesson). ≤500 lines; no library
  `unwrap`/`expect`.
- No `frf-*` engine change expected. **G1 (phase-30 topology) stays UNVERIFIED until this phase's live
  run observes `ice=connected`** (phase-30 lesson).

## First dispatch

`/kbd-apply p31-c001-prebuilt-image-runner`
