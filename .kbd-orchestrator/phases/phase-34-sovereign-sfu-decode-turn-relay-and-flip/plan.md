# Plan — phase-34-sovereign-sfu-decode-turn-relay-and-flip

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. The path is
> chosen (TURN on the phase-32 **bridge** stack — phase-33 decision) and **de-risked in the source**
> (str0m 0.21 accepts `typ relay`, `parser.rs:232`). No operator question; no engine change.

## Ordering rationale

Serial: **G1 → G2**. G1 adds a TURN relay so both browsers gather a **relay candidate** (a real IP
str0m pairs) — the environment-independent fix for phase-32's "no routable pair"; G2 is the decode
run + honest gate decision. The `SFU_MODE=sovereign` gate is decided only in c002, only on a real
`framesDecoded > 0`. **On the bridge stack — NOT host-net (phase-33 dead end).**

## Changes

### c001 — `p34-c001-coturn-turn-relay` (G1) · **agent: devops / e2e-runner on compose+harness**

**Goal:** upgrade coturn STUN-only → a **TURN relay** and give both browsers a `turn:` server so they
gather a relay candidate the SFU pairs.

- **coturn (`compose.sovereign.yml`):** drop `--stun-only`; add `--realm=frf`,
  `--use-auth-secret` + `--static-auth-secret=${TURN_SECRET}` (or `--user=frf:${TURN_SECRET}` +
  `--lt-cred-mech`), and `--external-ip=${TURN_EXTERNAL_IP}` / `--relay-ip` = coturn's reachable
  bridge address. Keep STUN. **Credential comes from env — never committed** (runner sets it; default
  a dev value only in the runner, not in compose source). `--min-port`/`--max-port` for the relay
  range if needed.
- **harness (`decode-probe.ts` + `media-decode.spec.ts`):** add `{ urls: 'turn:coturn:3478',
  username, credential }` to `iceServers` alongside the existing `stun:` in **both** the sender and
  receiver PCs. Thread `TURN_USERNAME`/`TURN_CREDENTIAL` (+ the existing `stunUrl`) through
  `DecodeProbeArgs` / the sender args from env. Keep the p32 Caddy TLS + in-network Playwright +
  DECODE_ONLY.
- **str0m:** accepts `typ relay` (confirmed) — **no engine change**; the run verifies the relay pair
  completes.
- **`MEDIA_ADVERTISE_IP` = the gateway bridge IP** (second host path).
- **runner:** set `TURN_SECRET`/`TURN_USERNAME`/`TURN_CREDENTIAL`/`TURN_EXTERNAL_IP`/`MEDIA_ADVERTISE_IP`
  for the run. File-size ≤500; no `frf-*` engine change.

**Exit:** the decode run's browsers gather a `turn`/relay candidate; the gateway accepts + pairs it;
the run reaches `ice=connected` + ≥1 session `state=Connected` + inbound `MediaData`/fan-out, or the
concrete blocker is diagnosed + recorded (→ CI pivot if TURN-on-bridge fails).

### c002 — `p34-c002-decode-run-and-flip` (G2) · **honest gate decision + CI-pivot check**

**Goal:** run the bridge+TURN decode; flip `SFU_MODE=sovereign` **only** on `framesDecoded > 0`.

- Run `scripts/run-media-decode.sh` (bridge + TURN); read the browser assertion + gateway str0m logs.
  Record `docs/PHASE-34-DECODE-RESULT.md` (`ice=` / `Connected` / `MediaData` / relay-candidate pairing).
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-34-SIGNOFF.md`.
- **Else:** re-affirm gated; if the relay pair still doesn't complete, **recommend the CI pivot
  (Target B)** — do not attempt further local network variants. Carry G3.
- Re-run the release gate; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c003 (conditional) — `p34-c003-carried-proofs-reaffirm` (G3) · **only if not folded into c002**

Re-affirm LiveKit x-node + admin-ui OIDC integration-gated. Likely folded into c002's SECURITY §6.

## Operational prerequisite for c002 (manual, via `!`)

```
! colima start --memory 8    # if the daemon is down
# gateway image is pre-built (p31 fail-fast enforces it)
```

## Discipline (carried 16→33)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0`; else gated with fresh rationale.
- Seed the openspec change dir up-front; read the QA verdict **and** the archive output; **update
  `progress.json` to N/N before any command mentioning the next stage** (phase-29). ≤500 lines; no
  library `unwrap`/`expect`; **TURN credential via env, never committed** (S1 no-secret gate).
- **NOT host-net** (phase-33 dead end). If TURN-on-bridge fails, **pivot to CI (Target B)** — no more
  local variants (phase-32/33 lesson).

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p34-c001-coturn-turn-relay | G1 | low-medium (coturn TURN + iceServers; no engine; str0m relay confirmed) | devops/e2e-runner |
| 2 | p34-c002-decode-run-and-flip | G2 | gate decision (+ CI-pivot rec) | — |
| 3 | p34-c003-carried-proofs-reaffirm (cond.) | G3 | low | — |

**First change to apply: `p34-c001-coturn-turn-relay`.**
