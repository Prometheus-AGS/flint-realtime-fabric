# Execution — phase-33-sovereign-sfu-decode-real-network-proof

> Date: 2026-07-09. Dispatch contract for the 2 planned changes. Backend: **OpenSpec** (KBD-owned
> per-task loop via `/kbd-apply`, never bare `/opsx:apply`). Target A (local host-net) locked.

## Backend selection

**`openspec`** — `openspec/` present; the phase updates `media-e2e` + `sfu-mode-consistency` per gate
decision. Each change is seeded as `openspec/changes/<id>/{proposal,tasks,specs/…}` **at the start of
its `/kbd-apply`** (seed-up-front lesson), driven one task per turn, QA-gated, verified + archived.

## Dispatch contract

| Order | Change | Tasks (walked one per turn) | QA gate | Archive |
|---|---|---|---|---|
| 1 | `p33-c001-host-net-decode-stack` | host-net gateway+playwright+caddy+coturn; `MEDIA_ADVERTISE_IP`=VM host IP; reconcile ports/extra_hosts/JWKS/Caddy/GATEWAY_URL; runner computes host IP | full (>3 files) | verify → archive |
| 2 | `p33-c002-decode-run-and-flip` | host-net decode run; DECODE-RESULT; conditional flip-or-reaffirm (+ TURN-fallback rec if no pair); SECURITY §6 + CHANGELOG + signoff | docs-heavy (read verdict + archive output) | verify → archive |

**Serial** — c002 must not begin until c001 is archived. c002 last (gate decision).

## Operational prerequisite for c002 (manual, operator runs via `!`)

```
! colima start --memory 8    # if the daemon is down
# gateway image is pre-built (p31 fail-fast enforces it)
```

## Fallback rule (carried from the plan)

If host-net still yields no routable pair, **fall back to TURN (Target C)** — do not re-peel bridge
variants (phase-32 conclusion). c002 records the TURN-fallback recommendation if warranted.

## Per-change QA gate

After each change reaches DONE: `.kbd-orchestrator/bin/qa-gate.sh <id>` → read the verdict **and**
the archive output. R1 check / R2-R3 clippy pedantic + unwrap_used / R4 fmt / R5 ≤500 lines / S1
no-secret / P1 openspec validate. ANY FAIL → mark BLOCKED + refine; do not archive.

## Invariants (carried 16→32)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` (c002); else gated with fresh
  rationale. `main.rs` gate untouched until then.
- Seed the change dir up-front; read verdict AND archive output; **update `progress.json` to N/N
  before any command mentioning the next stage** (phase-29). ≤500 lines; no library `unwrap`/`expect`.
- **No `frf-*` engine change** (compose/harness/runner only). When the environment is the blocker,
  pivot rather than peel (phase-32); host-net is that pivot, TURN the fallback.

## First dispatch

`/kbd-apply p33-c001-host-net-decode-stack`
