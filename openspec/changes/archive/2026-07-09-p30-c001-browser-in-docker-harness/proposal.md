# p30-c001-browser-in-docker-harness

## Why

Phase-29 B-topology: the decode browsers run via **host** Playwright and reach `localhost:28080`,
while the SFU runs in a container. On the Colima Linux-VM engine their ICE candidates live in
different address spaces (macOS host vs. VM bridge) — the STUN `srflx` reflects the VM view and the
gateway advertises host loopback, so no pair completes (`ice=disconnected`, phase-29). The fix is to
put the **browser inside the Docker network** so both endpoints share one address space.

## What Changes

- **Playwright/Chromium service** in `compose.sovereign.yml` (`mcr.microsoft.com/playwright:v1.52.0`,
  matching `@playwright/test ^1.52.0`) on the default compose network, with `admin-ui` mounted.
- **In-network gateway URL:** the spec reaches the gateway by service name (`http://gateway:8080` /
  `ws://gateway:8080`) via the existing `GATEWAY_URL` env — host runs unchanged.
- **Secure context:** Chromium is launched with
  `--unsafely-treat-insecure-origin-as-secure-origin=http://gateway:8080` so
  `getUserMedia`/`RTCPeerConnection`/`getStats` work on the in-network origin.
- **STUN service name:** `STUN_URL=stun:coturn:3478` so the in-network browser's srflx reflects the
  shared bridge.
- **Runner:** `scripts/run-media-decode.sh` runs the spec **inside** the Playwright container on the
  gateway+coturn network; host-side JWT mint + JWKS self-serve + Keto seed unchanged.

## Impact

- `compose.sovereign.yml` (playwright service), `admin-ui/e2e/media-decode.spec.ts` (secure-context
  launch flag), `scripts/run-media-decode.sh` (in-container run). No `frf-*` engine change.
