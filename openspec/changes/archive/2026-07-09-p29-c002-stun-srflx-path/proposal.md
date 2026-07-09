# p29-c002-stun-srflx-path

## Why

Phase-28 B1: Chrome emits its **host** candidates as mDNS `<uuid>.local` hostnames, which str0m's
`Candidate::from_sdp_string` cannot parse to an IP → every browser host candidate is dropped at the
gateway (`remoteCandidates=0`, ICE stuck at `new`). With the shared socket now in place (c001, the
gateway can host the two-peer room), the browser still needs a **routable** candidate str0m accepts.

## What Changes

- **coturn STUN** in `compose.sovereign.yml` (`3478/udp` published) so the decode run has a hermetic
  STUN server (operator decision: local coturn, not public STUN).
- **Harness `iceServers`:** the decode probe's `RTCPeerConnection` is configured with a STUN server
  (from `STUN_URL`, default `stun:127.0.0.1:3478`) so Chrome gathers a **server-reflexive** (`srflx`)
  candidate — a real IP that str0m parses and routes.
- **Defensive `.local` skip made explicit + tested:** c001's demux already skips an unparseable
  remote candidate (never fails the session); add a unit test proving a `.local` mDNS candidate
  string is skipped, not fatal.

## Impact

- `compose.sovereign.yml` (coturn service), `admin-ui/e2e/support/decode-probe.ts` +
  `webrtc-client.ts` (`iceServers`), a demux `.local`-skip test. No `MediaConfig`/port-contract
  change. Unblocks the decode proof (c003).
