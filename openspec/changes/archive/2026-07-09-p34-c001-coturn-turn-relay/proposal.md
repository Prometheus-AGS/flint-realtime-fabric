# p34-c001-coturn-turn-relay

## Why

Phase-32: on the bridge stack, `getUserMedia` works and ICE reaches `checking`, but no routable pair
forms (browser offers only mDNS `.local`; 0 srflx; gateway advertises `127.0.0.1` = the browser's own
loopback in-container). The standard, environment-independent fix is a **TURN relay**: a relay
candidate carries the relay server's real IP, always routes, and — confirmed in str0m 0.21
(`parser.rs:232` → `CandidateKind::Relayed`) — the SFU accepts + pairs it. No engine change.

## What Changes

- **coturn (`compose.sovereign.yml`):** drop `--stun-only`; run as a **TURN relay** — `--realm=frf`,
  `--use-auth-secret --static-auth-secret=${TURN_SECRET}` (long-term cred; from env, never committed),
  `--external-ip=${TURN_EXTERNAL_IP:-127.0.0.1}` (the relay address the in-network browser reaches).
  STUN stays available.
- **harness (`decode-probe.ts` + `media-decode.spec.ts`):** add `{ urls: turnUrl, username, credential
  }` to `iceServers` alongside the existing `stun:` in **both** the sender + receiver PCs; thread
  `turnUrl`/`turnUsername`/`turnCredential` through the args from env.
- **`MEDIA_ADVERTISE_IP` = the gateway bridge IP** (second host path).

## Impact

- `compose.sovereign.yml` (coturn TURN), `admin-ui/e2e/support/decode-probe.ts` +
  `media-decode.spec.ts` (iceServers turn:), `scripts/run-media-decode.sh` (TURN env). No `frf-*`
  engine change. TURN credential from env — never committed (S1).
