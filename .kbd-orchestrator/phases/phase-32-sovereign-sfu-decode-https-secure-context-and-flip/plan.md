# Plan — phase-32-sovereign-sfu-decode-https-secure-context-and-flip

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. The assessment
> settled G1 unambiguously (Caddy TLS sidecar; no operator decision, no engine change) — no
> clarifying question needed.

## Ordering rationale

Serial: **G1 → G2**. G1 gives the in-network browser a genuine `https://` secure context (so
`getUserMedia` works with no unsafe flags); G2 is the decode run + honest gate decision — the 9th
attempt, and the run where the whole media path is finally exercised end-to-end. The
`SFU_MODE=sovereign` gate is decided only in c002, only on a real `framesDecoded > 0`. **No `frf-*`
engine change** — a TLS *proxy* is not a gateway-transport change.

## Environment-pivot decision point (carried, explicit)

This is the 9th decode-proof attempt; the gate has held OFF through all 8 prior, on an unstable local
Colima VM. **If c002 still yields no decoded frame — whether a 10th harness layer or another VM crash
— stop peeling here and pivot the proof to a real CI runner / different host** rather than continue
locally. Record that recommendation in the c002 result + reflection. (Written into the phase goals.)

## Operational prerequisite for c002 (manual, operator runs via `!`)

The Colima daemon may be down (crashed twice in p31); the gateway image must exist (c001-p31 fail-fast
enforces it):

```
! colima start --memory 8         # (or restart) if the daemon is down
! docker compose -f compose.yml -f compose.sovereign.yml build gateway   # once, if the image is absent
```

## Changes

### c001 — `p32-c001-tls-sidecar-secure-context` (G1) · **agent: e2e-runner / devops on compose**

**Goal:** front the gateway with a **Caddy TLS sidecar** so the in-network browser reaches a genuine
`https://` (secure context) + `wss://` origin, and `navigator.mediaDevices`/`getUserMedia` work with
**no unsafe flags**.

- **compose:** add a `caddy` service to `compose.sovereign.yml` on the compose network with a minimal
  Caddyfile — auto internal self-signed cert, `reverse_proxy gateway:8080` (Caddy proxies HTTP **and**
  the WebSocket upgrade transparently), listening on `:8443`. `depends_on: gateway healthy`.
- **harness:** the Playwright service's `GATEWAY_URL` → `https://caddy:8443` (the spec derives
  `WS_URL = replace(/^http/, "ws")` → `wss://caddy:8443` automatically, and `goto`s the https origin).
- **secure-context flags:** in `media-decode.spec.ts`, launch Chromium with
  `--ignore-certificate-errors` (trust the self-signed) and **remove** the p31
  `--unsafely-treat-insecure-origin-as-secure-origin` / `--disable-features` /
  `--disable-site-isolation-trials` flags — a real `https://` context needs none. Keep the fake-media
  flags + STUN.
- **runner:** `scripts/run-media-decode.sh` brings up `caddy` too; the container run's `GATEWAY_URL`
  is set on the service (compose), host-side JWT/JWKS/Keto/health steps unchanged. File-size ≤500; no
  `frf-*` engine change.

**Exit:** the decode spec's browser has `navigator.mediaDevices` defined and `getUserMedia` succeeds;
the run proceeds to WS signaling + `create_session` (gateway str0m logs show a negotiated session), or
the concrete next blocker is diagnosed + recorded.

### c002 — `p32-c002-decode-run-and-flip` (G2) · **honest gate decision + environment-pivot check**

**Goal:** run the in-network decode over HTTPS; flip `SFU_MODE=sovereign` **only** on
`framesDecoded > 0`.

- Run `scripts/run-media-decode.sh`; read the browser assertion + gateway str0m logs. Record
  `docs/PHASE-32-DECODE-RESULT.md` (whether `ice=connected` / `state=Connected` / `MediaData` — the
  media path finally exercised end-to-end).
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-32-SIGNOFF.md`.
- **Else:** re-affirm gated with the fresh diagnostic detail (main.rs untouched); **and invoke the
  environment-pivot recommendation** (move the proof to CI/another host) if the blocker is again
  harness/environment rather than media. Carry G3.
- Re-run the release gate suite; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale (+ the
  pivot recommendation if warranted).

### c003 (conditional) — `p32-c003-carried-proofs-reaffirm` (G3) · **only if not folded into c002**

Re-affirm LiveKit x-node (G3.1) + admin-ui OIDC (G3.2) integration-gated in SECURITY §6 + CHANGELOG.
Likely folded into c002's SECURITY §6.

## Discipline (carried 16→31)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0`; else gated with fresh rationale.
- Seed the openspec change dir up-front; read the QA verdict **and** the archive output; **update
  `progress.json` to N/N before any command mentioning the next stage** (phase-29). ≤500 lines; no
  library `unwrap`/`expect`.
- **No more Chromium unsafe-origin flag chasing** — a real secure context (HTTPS) is the fix
  (phase-31 non-goal). **Stop live-iterating + pivot the environment** if c002 reveals another
  harness/VM layer (this phase's decision point).

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p32-c001-tls-sidecar-secure-context | G1 | medium (compose/harness; no engine) | e2e-runner |
| 2 | p32-c002-decode-run-and-flip | G2 | gate decision + pivot check | — |
| 3 | p32-c003-carried-proofs-reaffirm (cond.) | G3 | low | — |

**First change to apply: `p32-c001-tls-sidecar-secure-context`.**
