# Execution — phase-32-sovereign-sfu-decode-https-secure-context-and-flip

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`).

## Backend selection

**`openspec`** — `openspec/` present; the phase updates the `sfu-mode-consistency` + `media-e2e`
specs per gate decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}`
**at the start of its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated,
verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p32-c001-tls-sidecar-secure-context` | Caddy TLS sidecar in compose; `GATEWAY_URL=https://caddy:8443`; `--ignore-certificate-errors` + drop p31 unsafe flags; runner brings up caddy | full (>3 files) | verify → archive |
| 2 | `p32-c002-decode-run-and-flip` | decode run over HTTPS; DECODE-RESULT; conditional flip-or-reaffirm (+ env-pivot recommendation if another harness/VM layer); SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived. c002 last (gate decision + pivot check).

## Operational prerequisite for c002 (manual, operator runs via `!`)

```
! colima start --memory 8         # if the daemon is down (crashed twice in p31)
! docker compose -f compose.yml -f compose.sovereign.yml build gateway   # once, if the image is absent
```

## Environment-pivot decision point (carried)

9th decode attempt on an unstable local Colima VM. **If c002 yields no decoded frame — another harness
layer or a VM crash — stop peeling locally and recommend moving the proof to CI / another host** in
the c002 result + reflection.

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / S1
no-secret / P1 openspec validate. ANY FAIL → mark BLOCKED + refine; do not archive.

## Invariants (carried 16→31)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (phase-29). ≤500 lines; no library `unwrap`/`expect`.
- **No `frf-*` engine change** (a TLS proxy is not a gateway-transport change). **No more Chromium
  unsafe-origin flag chasing** — HTTPS is the fix (phase-31 non-goal). Stop + pivot if another
  harness/VM layer appears (this phase's decision point).

## First dispatch

`/kbd-apply p32-c001-tls-sidecar-secure-context`
