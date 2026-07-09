# Execution — phase-35-sovereign-sfu-decode-ci-linux-proof

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`). Target A: GitHub Actions `ubuntu-latest`.

## Backend selection

**`openspec`** — `openspec/` present; the phase updates `media-e2e` + `sfu-mode-consistency` per gate
decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of
its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated, verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p35-c001-ci-decode-job` | Linux-portable runner (172.17.0.1 JWKS; colima/HOST_NET gated); new `decode-proof` CI job (ubuntu-latest, workflow_dispatch, build+compose+decode+assert; secrets in-job) | full (>3 files) | verify → archive |
| 2 | `p35-c002-decode-run-and-flip` | trigger the CI job; DECODE-RESULT from job artifacts; conditional flip-or-reaffirm; SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived. c002 last (gate decision).

## Operational note for c002 (CI-triggered)

c002 runs on GitHub Actions. Trigger the `decode-proof` job via `workflow_dispatch` — either the
operator runs it, or I trigger it with the `gh` CLI if an authenticated `gh` is available in-session.
The job uploads gateway str0m logs + the Playwright report as artifacts for the honest result record.

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / **S1
no-secret (TURN/JWT secrets generated in-job, never committed)** / P1 openspec validate. ANY FAIL →
mark BLOCKED + refine; do not archive.

## Invariants (carried 16→34)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (phase-29). ≤500 lines; **no committed secrets**.
- **No `frf-*` engine change** (CI workflow + Linux address-adjust only). **No same-host
  macOS/Colima variants** (phase-34 non-goal).

## First dispatch

`/kbd-apply p35-c001-ci-decode-job`
