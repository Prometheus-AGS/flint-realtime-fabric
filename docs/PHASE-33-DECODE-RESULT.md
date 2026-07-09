# Phase-33 decode result (p33-c002)

> Date: 2026-07-09. The host-net stack (Target A, c001) was run. The stack boots most services, but the
> **gateway container goes `unhealthy` under `network_mode: host`** (a Colima host-net plumbing issue),
> so the decode never runs. **`framesDecoded=0`; `SFU_MODE=sovereign` stays gated OFF.** This is the
> 10th distinct blocker and it is Colima-host-net-specific — the point (per the phase-32/33 plan) to
> **stop peeling the local environment and take the recorded fallback/pivot**.

## Outcome: NOT proven — gate stays OFF (host-net gateway unhealthy)

```
[run-media-decode] HOST_NET: VM_IP=192.168.5.1 HOST_NAT_IP=192.168.5.2
Container flint-realtime-fabric-gateway-1  Error   dependency gateway failed to start
dependency failed to start: container …gateway-1 is unhealthy
RUNNER_EXIT=1
```

## What c001 achieved

- **The host-net override works structurally:** `!reset []` correctly cleared the inherited
  `ports:`/`extra_hosts` (the merge trap), the 3-file compose config is valid, and the runner derived
  the Colima addresses correctly (`VM_IP=192.168.5.1`, `HOST_NAT_IP=192.168.5.2`).
- **VM→macOS-host reachability is confirmed:** a probe showed the VM host-net **can** reach the
  macOS-served JWKS at `192.168.5.2:<port>` — so JWKS reachability is **not** the cause.

## The blocker: gateway `unhealthy` under `network_mode: host`

The gateway container reports **unhealthy**, which cascades (playwright `depends_on: gateway healthy`
→ "dependency failed to start"). The gateway's compose healthcheck is `curl -f
http://localhost:8080/readyz`. Under host-net this runs in the VM's shared network namespace rather
than an isolated one; the likely causes (not fully isolated, because the container was torn down by
the cleanup trap before inspection) are a **`8080` bind/namespace interaction under host-net** or a
**readyz dependency that host-net perturbs**. It is **not** JWKS reachability (confirmed reachable)
and **not** a media-path issue — the decode never started.

This is a **Colima host-net plumbing** problem — the 10th distinct blocker across phases 24→33, and
the third *environment* variant (bridge → secure-context → host-net) that fails in its own new way.

## Decision — honest gate held + take the recorded pivot (do NOT keep peeling host-net)

`framesDecoded == 0`; the gateway never became healthy; no session negotiated. **`SFU_MODE=sovereign`
is NOT flipped.** `main.rs` untouched; SECURITY §6 keeps the plane *composed but not proven*.

**This is the decision point the phase-32 reflection and phase-33 plan explicitly drew:** host-net was
the chosen Target A because it was the lowest-effort test of a routable pair — but it introduced its
own Colima-VM plumbing failure (gateway health under host-net) before the media exchange could even
run. Per the plan's fallback rule ("if host-net still yields no routable pair, fall back to TURN
(Target C); do not re-peel bridge variants"), and the phase-32 lesson ("when the environment itself is
the blocker, pivot"), the honest move is **not** to debug Colima host-net health semantics further.

## Recommended next — the fallback / pivot (operator decision)

Two recorded paths, both avoid further same-host-VM plumbing:

1. **Target C — local + TURN relay (recommended next local step):** keep the *bridge* stack that
   already boots healthy (phase-32 got ICE to `checking`), and add a **TURN relay** — coturn with
   `--external-ip` + realm + long-term creds; harness `iceServers` adds the `turn:` URL. A **relay
   candidate always routes** regardless of host/mDNS/srflx/loopback topology. This directly fixes the
   phase-32 "no routable pair" without host-net's health problem.
2. **Target B — CI / real Linux host:** run the whole proof (containerized rig) on `ubuntu-latest`
   where host networking is native and healthy — the durable home.

The media path, shared-socket demux, `.local` skip, and secure context are all proven; the residual is
purely a routable candidate pair, and **TURN (C) is the standard, environment-independent answer** —
it should be the next change rather than more host-net debugging.
