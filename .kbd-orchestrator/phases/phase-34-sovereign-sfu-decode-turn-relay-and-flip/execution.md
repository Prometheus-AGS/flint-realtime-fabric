# Execution — phase-34-sovereign-sfu-decode-turn-relay-and-flip

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`). TURN on the bridge; str0m relay support
> confirmed.

## Backend selection

**`openspec`** — `openspec/` present; the phase updates `media-e2e` + `sfu-mode-consistency` per gate
decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of
its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated, verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p34-c001-coturn-turn-relay` | coturn STUN→TURN (realm + static-auth-secret + external-ip); harness `iceServers` turn: + creds (both PCs, via env); `MEDIA_ADVERTISE_IP`=gateway bridge IP; runner env | full (>3 files) | verify → archive |
| 2 | `p34-c002-decode-run-and-flip` | bridge+TURN decode run; DECODE-RESULT; conditional flip-or-reaffirm (+ CI-pivot rec if relay pair fails); SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived. c002 last (gate decision).

## Operational prerequisite for c002 (manual, operator runs via `!`)

```
! colima start --memory 8    # if the daemon is down
# gateway image is pre-built (p31 fail-fast enforces it)
```

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / **S1
no-secret (the TURN credential MUST come from env, not committed)** / P1 openspec validate. ANY FAIL →
mark BLOCKED + refine; do not archive.

## Invariants (carried 16→33)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (phase-29). ≤500 lines; no library `unwrap`/`expect`;
  **TURN credential via env, never committed**.
- **No `frf-*` engine change** (str0m accepts `typ relay`, confirmed). **Bridge stack, NOT host-net**
  (phase-33 dead end). If TURN-on-bridge fails → **CI pivot (Target B)**, no more local variants.

## First dispatch

`/kbd-apply p34-c001-coturn-turn-relay`
