# Plan — phase-30-sovereign-sfu-decode-topology-and-flip

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. Operator
> decision locked this turn: **G1 fix = browser inside the Docker network** (the only robust fix on
> the Colima Linux-VM engine; host-networking and host-side srflx both fail the host↔VM split).

## Ordering rationale

Serial: **G1 → G2**. G2 (decode + flip) is untestable until the browser and SFU share one network
(G1). c001 is the harness/compose restructure that removes the host↔VM split; c002 is the decode
run + the honest gate decision. The `SFU_MODE=sovereign` gate is decided only in c002, only on a real
`framesDecoded > 0`. The SFU engine is already proven correct (phase-29) — **no `frf-*` engine change
is planned**; if a genuine engine issue surfaces once a pair completes, ADR it then.

## Changes

### c001 — `p30-c001-browser-in-docker-harness` (G1) · **agent: e2e-runner / rust-build-resolver on compose**

**Goal:** run the decode browsers **inside the compose network** so their ICE candidates and the
gateway's `0.0.0.0:40000` share the Colima VM bridge, and a pair completes to `Connected`.

- **compose:** add a **Playwright/Chromium service** to `compose.sovereign.yml`
  (`mcr.microsoft.com/playwright:v1.52.0`, matching `@playwright/test ^1.52.0`) on the default
  compose network, with the admin-ui e2e mounted/available and the browser deps present.
- **in-network gateway URL:** the spec reaches the gateway by **service name** (`http://gateway:8080`
  + `ws://gateway:8080`) instead of `localhost:28080`. Parameterize via the existing `GATEWAY_URL`
  env so the container run sets the in-network URL and the host run is unchanged.
- **secure context:** `navigator.mediaDevices`/`RTCPeerConnection` need a secure context; an
  in-network `http://gateway:8080` origin is not `localhost`. Launch Chromium with
  `--unsafely-treat-insecure-origin-as-secure-origin=http://gateway:8080` (+
  `--disable-features=...` as needed) so `getUserMedia`/`getStats` work. Keep the existing
  `--use-fake-*-for-media-stream` flags.
- **runner:** `scripts/run-media-decode.sh` runs the spec **inside** the Playwright container
  (`docker compose run playwright …` or an exec) on the same network as gateway + coturn; keep the
  RS256 JWKS self-serve + Keto seed (now reachable in-network), STUN via coturn service name.
- **coturn:** point `STUN_URL` at the coturn **service name** (`stun:coturn:3478`) so the in-network
  browser's srflx reflects the shared bridge.
- File-size ≤500; no library `unwrap`/`expect` (no Rust change expected). Keep the host-run path as a
  documented fallback.

**Exit:** the decode run reaches `ice=connected` and ≥1 session logs `state=Connected` + inbound
`MediaData` at the gateway (fan-out begins), or the concrete blocker is diagnosed + recorded.

### c002 — `p30-c002-decode-run-and-flip` (G2) · **honest gate decision**

**Goal:** re-run the in-network decode proof; flip `SFU_MODE=sovereign` **only** on
`framesDecoded > 0`.

- Run the in-network `scripts/run-media-decode.sh`; read the browser assertion + gateway str0m logs.
  Record `docs/PHASE-30-DECODE-RESULT.md`.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-30-SIGNOFF.md`.
- **Else:** re-affirm gated with the fresh diagnostic detail (main.rs untouched). Carry G3.
- Re-run the release gate suite; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c003 (conditional) — `p30-c003-carried-proofs-reaffirm` (G3) · **only if not folded into c002**

Re-affirm LiveKit x-node (G3.1) + admin-ui OIDC (G3.2) integration-gated in SECURITY §6 + CHANGELOG.
Likely folded into c002's SECURITY §6 edit — emit as a change only if it needs its own diff.

## Discipline (carried 16→30)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` against a live gateway; else
  gated off with fresh rationale. No "healthy but does nothing."
- **Seed the openspec change dir at the START of every `/kbd-apply`.** Read the QA verdict **and**
  the archive output. **Update `progress.json` to N/N before any command mentioning the next stage**
  (pipeline-enforce guards on the pre-update snapshot — phase-29 lesson). File-size ≤500; no library
  `unwrap`/`expect`; clippy pedantic + `deny(warnings)`.

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p30-c001-browser-in-docker-harness | G1 | **high** (harness/compose restructure; secure-context) | e2e-runner |
| 2 | p30-c002-decode-run-and-flip | G2 | gate decision | — |
| 3 | p30-c003-carried-proofs-reaffirm (cond.) | G3 | low | — |

**First change to apply: `p30-c001-browser-in-docker-harness`.**
