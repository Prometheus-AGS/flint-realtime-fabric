# Execution — phase-29-sovereign-sfu-shared-socket-and-stun

> Date: 2026-07-09. Dispatch contract for the 3 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`).

## Backend selection

**`openspec`** — `openspec/` present; the phase needs spec-backed traceability (the
`sfu-mode-consistency` spec is updated per gate decision), and the media plane is under
ADR-005/006/007 (c001 adds ADR-008). Each change is seeded as
`openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of its `/kbd-apply`** (seed-up-front
lesson), driven one task per turn, QA-gated, then verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p29-c001-shared-demux-socket` | ADR-008; shared-socket refactor + `demux.rs` split; multi-session in-process test | full (Rust; >3 files) | openspec verify → archive |
| 2 | `p29-c002-stun-srflx-path` | coturn in compose; harness `iceServers`; defensive `.local` skip + test | full | verify → archive |
| 3 | `p29-c003-decode-run-and-flip` | decode re-run; DECODE-RESULT; conditional flip-or-reaffirm; SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Strict serial** — c002 must not begin until c001 is archived (B1 untestable until the shared
socket exists). c003 last (gate decision).

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / S1
no-secret / P1 openspec validate. ANY FAIL → mark BLOCKED + refine; do not archive.

## Invariants (carried 16→29)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c003); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; ≤500 lines; no library
  `unwrap`/`expect`; clippy pedantic + `deny(warnings)`; `tracing` spans across port boundaries.
- One-port-per-adapter + clean-arch dependency rule intact; ADR-008 refines (not re-opens)
  ADR-005/006.

## First dispatch

`/kbd-apply p29-c001-shared-demux-socket`
