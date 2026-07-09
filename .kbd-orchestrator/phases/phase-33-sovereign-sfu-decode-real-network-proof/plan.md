# Plan — phase-33-sovereign-sfu-decode-real-network-proof

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. Operator decision
> locked this turn: **Target A — local host-net (both gateway + browser as `network_mode: host`
> containers on the Colima VM)**. Lowest-effort path to a routable candidate pair; no CI, no TURN.

## Ordering rationale

Serial: **G1 → G3** (G2 TURN is not needed for Target A — host candidates pair when both peers share
one net stack; TURN stays a documented fallback). G1 puts the gateway + browser on the VM's host
network so their ICE candidates share one address space; G3 is the decode run + honest gate decision.
The `SFU_MODE=sovereign` gate is decided only in c002, only on a real `framesDecoded > 0`. **No `frf-*`
engine change** — compose/harness/runner only.

## Host-net implications to reconcile (Target A)

With `network_mode: host` on the Colima VM:
- **`ports:` mappings + `extra_hosts` become no-ops** — the gateway binds the VM host net directly
  (`8080`, `40000/udp`); the Caddy sidecar (`8443`) + coturn (`3478`) also bind the VM host net, so
  everything is reachable by `localhost`/the VM IP within the VM.
- **`host.docker.internal` won't resolve under host-net** — the host-served JWKS must be reached via
  the VM host IP (or `127.0.0.1` since the gateway is now on the VM host net). `GATEWAY_JWKS_URL` +
  the harness `GATEWAY_URL` shift to the VM-host address.
- **`MEDIA_ADVERTISE_IP` = the VM host IP** (not `127.0.0.1`) so the gateway's advertised host
  candidate is the address the (also-host-net) browser reaches — the two peers pair.
- **coturn STUN still helps** (browser gets a srflx on the same net) but host candidates should now
  pair directly; keep coturn, no TURN.

## Changes

### c001 — `p33-c001-host-net-decode-stack` (G1) · **agent: devops / e2e-runner on compose**

**Goal:** run the gateway + Playwright browser (+ coturn, + Caddy) on the **VM host network** so
browser and SFU share one routable stack and host candidates pair.

- **compose:** put `gateway`, `playwright`, `caddy`, `coturn` on `network_mode: host` (a dedicated
  override, or extend `compose.sovereign.yml`). Drop the now-redundant `ports:`/`extra_hosts` for
  host-net services. Reconcile the Caddy `--from https://<host>:8443 --to http://localhost:8080` and
  the harness `GATEWAY_URL=https://<host>:8443` to host-net addresses.
- **`MEDIA_ADVERTISE_IP` = the VM host IP** (resolve it in the runner — e.g. the default-route IP —
  and pass it), so the advertised host candidate is browser-reachable.
- **runner:** `scripts/run-media-decode.sh` computes the VM host IP, sets `GATEWAY_JWKS_URL` +
  `GATEWAY_URL` + `MEDIA_ADVERTISE_IP` for host-net, brings the stack up. Keep JWT mint + JWKS serve +
  Keto seed (now reached via the VM host IP, not `host.docker.internal`). File-size ≤500; no `frf-*`
  engine change.

**Exit:** the decode run's browser + gateway are on one network; the run reaches `ice=connected` +
≥1 session `state=Connected` + inbound `MediaData`/fan-out at the gateway, or the concrete blocker is
diagnosed + recorded (with TURN as the documented fallback if host candidates still don't pair).

### c002 — `p33-c002-decode-run-and-flip` (G3) · **honest gate decision**

**Goal:** run the host-net decode; flip `SFU_MODE=sovereign` **only** on `framesDecoded > 0`.

- Run `scripts/run-media-decode.sh`; read the browser assertion + gateway str0m logs. Record
  `docs/PHASE-33-DECODE-RESULT.md` (whether `ice=connected` / `Connected` / `MediaData`).
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-33-SIGNOFF.md`.
- **Else:** re-affirm gated with the concrete detail; if host candidates still don't pair, recommend
  the **TURN fallback** (Target C) as the next change. Carry G4.
- Re-run the release gate; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c003 (conditional) — `p33-c003-carried-proofs-reaffirm` (G4) · **only if not folded into c002**

Re-affirm LiveKit x-node (G4.1) + admin-ui OIDC (G4.2) integration-gated. Likely folded into c002's
SECURITY §6.

## Operational prerequisite for c002 (manual, via `!`)

```
! colima start --memory 8    # if the daemon is down
# gateway image is pre-built (p31 c001 fail-fast enforces it; rebuild only if source changed)
```

## Discipline (carried 16→32)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0`; else gated with fresh rationale.
- Seed the openspec change dir up-front; read the QA verdict **and** the archive output; **update
  `progress.json` to N/N before any command mentioning the next stage** (phase-29). ≤500 lines; no
  library `unwrap`/`expect`.
- If host-net still yields no routable pair, **fall back to TURN (Target C)** — do not re-peel bridge
  variants (phase-32 conclusion).

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p33-c001-host-net-decode-stack | G1 | medium (host-net compose reconcile; no engine) | devops/e2e-runner |
| 2 | p33-c002-decode-run-and-flip | G3 | gate decision (+ TURN-fallback rec) | — |
| 3 | p33-c003-carried-proofs-reaffirm (cond.) | G4 | low | — |

**First change to apply: `p33-c001-host-net-decode-stack`.**
