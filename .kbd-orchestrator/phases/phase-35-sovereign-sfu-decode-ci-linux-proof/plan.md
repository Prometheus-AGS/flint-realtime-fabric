# Plan — phase-35-sovereign-sfu-decode-ci-linux-proof

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. Operator decision
> locked: **Target A — GitHub Actions `ubuntu-latest`** (a net-new decode job in `ci.yml`; off the
> local box). Native host networking on Linux removes the six candidate-address confusions.

## Ordering rationale

Serial: **G1/G2 → G3**. G1 makes the rig Linux/CI-portable + adds the CI decode job; G2 is that job
running the decode on `ubuntu-latest`; G3 is the honest gate decision. The `SFU_MODE=sovereign` gate
is decided only in c002, only on a real `framesDecoded > 0`. **No `frf-*` engine change** — CI
workflow + Linux address-adjustment only. **No same-host macOS/Colima variants** (hard non-goal).

## Changes

### c001 — `p35-c001-ci-decode-job` (G1/G2) · **agent: devops / e2e-runner on CI + runner**

**Goal:** a reproducible **GitHub Actions decode job** that boots the sovereign stack on
`ubuntu-latest` (native host networking) and runs the Playwright decode.

- **Linux-portability in the runner** (`scripts/run-media-decode.sh`): gate the macOS/Colima bits
  behind the existing `HOST_NET`/colima branches so the default Linux path uses **`172.17.0.1`** for
  the gateway→JWKS URL (docker0 bridge gateway; replaces `host.docker.internal`), and needs no
  `colima ssh`. A `CI=1`/Linux auto-detect (or explicit env) selects the Linux host address. Keep the
  bridge stack + Caddy TLS + coturn STUN/TURN + in-network Playwright + DECODE_ONLY (all already
  present).
- **CI workflow** (`.github/workflows/`): a **new `decode-proof` job** (own file or a `ci.yml` job) on
  `ubuntu-latest` — checkout → `setup-protoc` + rust toolchain (for the gateway build) + node/pnpm →
  **build the gateway image** (16 GB runner, no OOM) → run the decode entrypoint (which boots the
  stack via `docker compose` and runs the Playwright spec, browsers via the `mcr` image or an install
  step) → **assert `framesDecoded > 0`** (job fails if not). `MEDIA_ADVERTISE_IP` = the runner's real
  IP; TURN secret via a generated env (S1 — no committed secret); STUN/TURN belt-and-suspenders.
- Trigger: `workflow_dispatch` (manual) + optionally on a label — **do not** run the heavy decode on
  every push. File-size ≤500; no `frf-*` engine change.

**Exit:** the `decode-proof` job boots the full stack on `ubuntu-latest` and runs the decode spec
end-to-end (reaches the browser↔gateway media exchange with logs), or the concrete blocker is
diagnosed + recorded.

### c002 — `p35-c002-decode-run-and-flip` (G3) · **honest gate decision**

**Goal:** run the CI decode job; flip `SFU_MODE=sovereign` **only** on `framesDecoded > 0`.

- Run the `decode-proof` job (via `workflow_dispatch`); read the job's browser assertion + gateway
  str0m logs (uploaded as an artifact). Record `docs/PHASE-35-DECODE-RESULT.md`.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-35-SIGNOFF.md`.
- **Else:** re-affirm gated with the fresh CI diagnostics (main.rs untouched). Carry G4.
- Re-run the release gate; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c003 (conditional) — `p35-c003-carried-proofs-reaffirm` (G4) · **only if not folded into c002**

Re-affirm LiveKit x-node + admin-ui OIDC integration-gated. Likely folded into c002's SECURITY §6.

## Operational note

c002 runs on GitHub Actions — the operator triggers the `workflow_dispatch` run (or I trigger it if a
`gh` CLI with auth is available in-session; otherwise the operator runs it and shares the result). The
job uploads gateway logs + the Playwright report as artifacts for the honest result record.

## Discipline (carried 16→34)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0`; else gated with fresh rationale.
- Seed the openspec change dir up-front; read the QA verdict **and** the archive output; **update
  `progress.json` to N/N before any command mentioning the next stage** (phase-29). ≤500 lines; **no
  committed secrets (S1)** — TURN/JWT secrets generated in-job.
- **No `frf-*` engine change.** **No same-host macOS/Colima variants** (phase-34 non-goal).

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p35-c001-ci-decode-job | G1/G2 | medium (CI workflow + Linux address-adjust; no engine) | devops/e2e-runner |
| 2 | p35-c002-decode-run-and-flip | G3 | gate decision (CI-triggered) | — |
| 3 | p35-c003-carried-proofs-reaffirm (cond.) | G4 | low | — |

**First change to apply: `p35-c001-ci-decode-job`.**
