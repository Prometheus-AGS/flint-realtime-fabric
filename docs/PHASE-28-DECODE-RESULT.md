# Phase-28 decode result (p28-c003)

> Date: 2026-07-09. The decode proof re-run with the p28-c002 ICE candidate-IP fix in place. The
> `0.0.0.0` bug is fixed; two further, concrete media-path defects are now visible. **No decoded
> frame — `SFU_MODE=sovereign` stays gated off.** Evidence below, not a guess.

## Outcome: NOT proven — gate stays OFF

```
framesDecoded=0 bytes=0 reason=timeout ice=new localCandidates=2 remoteCandidates=0
1 failed  ·  RUNNER_EXIT=1
```

## What the p28-c002 fix DID achieve (real progress)

The gateway now constructs a **valid** host candidate — the `0.0.0.0` rejection is gone:

```
frf_media_str0m::session: sovereign: session negotiated (offer accepted, host candidate advertised)
  bind=0.0.0.0:40000 advertised=candidate:… 1 udp 2130706175 127.0.0.1 40000 typ host
frf_media_str0m::driver: sovereign: connection state … state=Connecting
```

`create_session` succeeds, the session negotiates, and str0m advertises `127.0.0.1:40000`. This is
strictly further than every prior run (which failed at candidate construction).

## The two NEW blockers the run surfaced

### B1 — Chrome mDNS `.local` host candidates are rejected

```
frf_media_str0m::driver: bad remote candidate — skipping
  error=ICE bad candidate: candidate:… 1 udp 2113937151 <uuid>.local 56166 typ host …
         bad address: invalid IP address syntax
```

Chrome, for privacy, emits its **host** candidates as mDNS hostnames (`<uuid>.local`) rather than
raw LAN IPs. str0m's `Candidate::parse`/`add_remote_candidate` rejects a non-IP address. So **every
one of the browser's host candidates is dropped** at the gateway. The browser's `localCandidates=2`
are exactly these `.local` host candidates; with them rejected, there is no candidate pair to check
from the gateway side, and (combined with B2) ICE never advances. The gateway's own `127.0.0.1`
candidate does reach the browser envelope path, but the harness reports `remoteCandidates=0` — see
B2 for why the session that would relay it is already dead.

### B2 — Fixed single UDP port can't bind per-session (`EADDRINUSE`)

```
frf_gateway::media_bridge: sovereign: create_session failed
  error=transport error: media transport error: udp bind: Address already in use (os error 98)
frf_gateway::media_bridge: sovereign: add_remote_candidate failed error=… unknown session
```

The transport binds `MEDIA_BIND_ADDR:MEDIA_UDP_PORT` (`0.0.0.0:40000`) **per session**. The harness
opens the WS, negotiates once (that session binds 40000), and a second negotiate — or a
re-connect/second peer in the same run — tries to bind 40000 again and fails with `EADDRINUSE`. With
no session created, the browser's subsequent trickle candidates all hit **"unknown session"**, so
none are added and `remoteCandidates` stays 0. A single fixed media port is a single-session design;
it cannot host the two-peer room the decode proof drives.

## Root-cause summary

Neither is a fan-out or DTLS bug. They are the **standard browser↔SFU ICE realities** the in-process
loopback proof never exercised:

- **B1:** browsers hide host IPs behind mDNS — an SFU must either resolve `.local` (mDNS query) or,
  more practically, rely on **srflx/relay** candidates (STUN/TURN) rather than the browser's host
  candidate. On loopback there are no `.local` candidates, so this never appeared until a real
  browser connected.
- **B2:** one UDP socket per fixed port serves one session; a real SFU uses **one shared demuxing
  socket** (route by ICE ufrag/remote 5-tuple) or a **per-session ephemeral port**, not a single
  fixed port re-bound per session.

Both are legitimate str0m-SFU engineering items (str0m's own examples use a single shared UDP socket
that demuxes by `Rtc`), not harness noise.

## Decision — honest gate held (operator-confirmed discipline)

`framesDecoded == 0`. Per the discipline carried since phase-16 and re-confirmed by the operator
twice (phase-24, phase-27): **`SFU_MODE=sovereign` is NOT flipped.** `main.rs` keeps the gate-off
warning; SECURITY §6 keeps the media plane marked *composed but not proven*. The advance is that the
gateway-side candidate is now valid and the remaining work is two well-identified transport-design
items (shared demuxing socket + mDNS/srflx handling), carried to the next phase.

## Next (carried)

1. **B2 first** (blocks everything): one shared demuxing UDP socket in `StrOmTransport`, routing
   inbound datagrams to the owning `Rtc` by remote 5-tuple / ICE ufrag — no per-session re-bind.
2. **B1:** add a STUN server-reflexive path (or mDNS resolution) so the browser's usable candidate
   reaches the gateway; the `.local` host candidate alone is not routable.
3. Re-run the decode proof; flip only on `framesDecoded > 0`.
