# p33-c001-host-net-decode-stack

## Why

Phase-32: with the browser in a bridge container, its ICE candidates and the SFU's never shared a
routable address (0 srflx, `.local` skipped, gateway advertised `127.0.0.1` = the browser's own
loopback in-container). Operator decision (Target A): run **both** the gateway and the Playwright
browser (+ caddy + coturn) on the **VM host network** so they share one real network stack — host
candidates then pair directly (Colima is itself a Linux VM; `network_mode: host` shares the VM's net).

## What Changes

- **`compose.host-net.yml`** (a new override layered on `compose.yml` + `compose.sovereign.yml`) puts
  `gateway`, `playwright`, `caddy`, `coturn` on `network_mode: host`. Under host-net, service-name DNS
  and `ports:`/`extra_hosts` are inert — every service is reachable at `localhost:<port>` within the
  VM. So: gateway binds `0.0.0.0:8080` + `40000/udp` on the VM host net; `MEDIA_ADVERTISE_IP=127.0.0.1`
  now works (the browser shares the same loopback); Caddy `--to http://localhost:8080`; harness
  `GATEWAY_URL=https://localhost:8443`; `STUN_URL=stun:localhost:3478`; `GATEWAY_JWKS_URL` →
  `http://localhost:<JWKS_PORT>` (no `host.docker.internal`).
- **Runner:** `scripts/run-media-decode.sh` layers the host-net override and sets the host-net env.

## Impact

- `compose.host-net.yml` (new), `scripts/run-media-decode.sh` (layer + env). No `frf-*` engine change,
  no change to `compose.sovereign.yml` (host-net is an additive override).
