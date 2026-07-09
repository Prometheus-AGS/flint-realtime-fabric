# Execution — phase-30-sovereign-sfu-decode-topology-and-flip

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`). Decision locked: **browser-in-Docker**.

## Backend selection

**`openspec`** — `openspec/` present; the phase updates the `sfu-mode-consistency` spec per gate
decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of
its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated, verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p30-c001-browser-in-docker-harness` | Playwright/Chromium service in compose; in-network `GATEWAY_URL` + secure-context flag; runner runs spec in-container; `STUN_URL=stun:coturn:3478` | full (>3 files) | verify → archive |
| 2 | `p30-c002-decode-run-and-flip` | decode re-run in-network; DECODE-RESULT; conditional flip-or-reaffirm; SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived (G2 untestable until the browser + SFU share
one network). c002 last (gate decision).

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / S1
no-secret / P1 openspec validate. ANY FAIL → mark BLOCKED + refine; do not archive.

## Invariants (carried 16→30)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (pipeline-enforce guards the pre-update snapshot).
  ≤500 lines; no library `unwrap`/`expect`; clippy pedantic + `deny(warnings)`.
- Engine proven correct (phase-29) — **no `frf-*` engine change expected**; ADR any real engine
  change if one surfaces once a pair completes.

## First dispatch

`/kbd-apply p30-c001-browser-in-docker-harness`
