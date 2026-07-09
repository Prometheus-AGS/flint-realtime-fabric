# Phase-33 signoff — sovereign SFU decode: host networking attempt

> Date: 2026-07-09. Phase-33 tried host networking (Target A) to give the browser + SFU one routable
> stack. The override is structurally correct, but the **gateway goes `unhealthy` under
> `network_mode: host`** (a Colima host-net plumbing issue) before the decode runs.
> **`SFU_MODE=sovereign` stays gated OFF** — and the recorded next step is a **TURN relay (Target C)**,
> not more host-net debugging.

## Gate decision: OFF (honest gate held) + TURN pivot recommended

No `framesDecoded > 0`; the gateway never became healthy; no session negotiated. Per the phase-16→32
discipline: the gate does not flip until a real receiver observes a decoded frame. `main.rs` untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p33-c001 | `compose.host-net.yml` (`network_mode: host` via `!reset`) + runner `HOST_NET=1` with Colima IP derivation | none (harness) |
| p33-c002 | Host-net decode run + honest gate decision + TURN-pivot recommendation | **held OFF** |

## Evidence

- **c001 structurally correct:** `!reset` cleared merge-inherited `ports:`; compose config valid;
  runner derived `VM_IP=192.168.5.1`, `HOST_NAT_IP=192.168.5.2`; VM→macOS-host JWKS reachability
  **confirmed** (so JWKS is not the cause).
- **Blocker:** the gateway container is **`unhealthy` under `network_mode: host`** (its `/readyz` /
  `8080` healthcheck), cascading to the browser via `depends_on: gateway healthy`. Not JWKS, not
  media — the decode never started. `framesDecoded=0`.

## The decision — take the recorded pivot, don't keep peeling host-net

This was the 10th distinct blocker and the third *environment* variant (bridge → secure-context →
host-net) to fail in its own new way. Per the phase-32/33 plan's fallback rule and the "pivot when the
environment is the blocker" lesson, the honest move is **not** to debug Colima host-net health
semantics further.

## Carried to next phase — TURN relay (Target C), CI the durable home (Target B)

1. **Target C (recommended next):** on the **bridge** stack that already boots healthy, add a **TURN
   relay** — coturn `--external-ip` + realm + long-term creds; harness `iceServers` adds the `turn:`
   URL. A relay candidate always routes regardless of host/mDNS/srflx/loopback topology — the standard,
   environment-independent fix for the phase-32 "no routable pair".
2. **Target B (durable home):** run the containerized proof on `ubuntu-latest` CI where host
   networking is native and healthy.

The media path, shared-socket demux, `.local` skip, and secure context are all proven; the residual is
purely a routable candidate pair — and TURN is the standard answer.

## Verification

- Host `cargo test -p frf-media-str0m` (31), `cargo fmt --check` — green (no engine change).
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the TURN pivot.
