# Phase-32 decode result (p32-c002)

> Date: 2026-07-09. The HTTPS secure-context fix (c001) **worked** — `getUserMedia` succeeds, sessions
> negotiate, and ICE reaches `checking` for the first time in the whole 24→32 sequence. But
> `framesDecoded=0`: the browser produces **only mDNS `.local` host candidates** (no usable STUN
> `srflx`), which str0m rejects, so ICE stalls at `checking` → `Disconnected`. **`SFU_MODE=sovereign`
> stays gated OFF.** This is a genuine ICE-connectivity/networking blocker, not a harness layer.

## Outcome: NOT proven — gate stays OFF (ICE connectivity)

```
framesDecoded=0 bytes=0 reason=timeout ice=checking localCandidates=4 remoteCandidates=1
1 failed  ·  RUNNER_EXIT=1
```

## Breakthrough: the secure-context fix (c001) worked — furthest point yet

The phase-31 `getUserMedia` error is **gone**. The gateway log shows real end-to-end negotiation for
the first time:

```
str0m::create_session … sovereign: session negotiated (offer accepted, host candidate advertised)
  advertised=candidate:… 127.0.0.1 40000 typ host
frf_media_str0m::driver: sovereign: connection state … state=Connecting
```

- ✅ **`getUserMedia` works** — the sender acquires a track and offers (HTTPS secure context via the
  Caddy sidecar did exactly what it was for).
- ✅ **ICE advanced to `checking`** (browser) with `remoteCandidates=1` — up from phase-30's
  `disconnected`/`new`. This is the furthest ICE has ever progressed.
- ✅ **Sessions negotiate + the shared-socket demux routes** (multiple sessions on `0.0.0.0:40000`, no
  `EADDRINUSE`) — the phase-29 B2 fix holds under a real browser.

## The blocker: no usable STUN srflx candidate → ICE stalls at `checking`

Gateway log evidence: **0 `srflx` candidates, 12 `.local` host candidates skipped**, 0 `Connected`, 0
`MediaData`:

```
frf_media_str0m::demux: bad remote candidate — skipping
  error=ICE bad candidate: candidate:… <uuid>.local … typ host … bad address: invalid IP address syntax
frf_media_str0m::driver: sovereign: connection state … state=Disconnected
```

The browser gathers **only mDNS `<uuid>.local` host candidates** — it is **not** producing a STUN
server-reflexive (`srflx`) candidate, even though the harness sets `iceServers: [stun:coturn:3478]`
(p29-c002) and coturn is up. So the only candidates that reach the gateway are the unroutable `.local`
names str0m correctly rejects; with no usable pair, ICE checks and fails (`checking` → `Disconnected`).

### Why (evidence-led)

`remoteCandidates=1` on the browser = the gateway's `127.0.0.1:40000` host candidate arrived — but
`127.0.0.1` inside the Playwright container is the **browser itself**, not the gateway (the gateway's
real in-network address is its bridge IP, e.g. `gateway:40000`). And the browser's own srflx isn't
being gathered/relayed — coturn in `--stun-only` mode may not be reflecting a usable address on the
Docker bridge, or the srflx candidate is being filtered before it reaches the gateway. Either way, the
two peers never share a routable pair.

This is the **same B1 candidate-topology class as phase-29**, now isolated cleanly (the secure-context
noise is gone): the in-network browser and the SFU need to exchange **routable** candidates, and the
current setup yields only mDNS `.local` (browser) + host-loopback `127.0.0.1` (gateway) — neither
pairs.

## Decision — honest gate held + ENVIRONMENT-PIVOT recommended

`framesDecoded == 0`, no session reached `Connected`, no `MediaData`. **`SFU_MODE=sovereign` is NOT
flipped.** `main.rs` untouched; SECURITY §6 keeps the plane *composed but not proven*.

**This is the environment-pivot decision point** (written into the phase plan/goals). Two things are
now true: **(a)** the media-path *code* is proven correct as far as any in-process test can show —
negotiate, shared-socket demux, `.local` skip, secure context all work live; and **(b)** the residual
is a **candidate-topology/networking** problem that this same-host Colima-VM Docker setup keeps
re-surfacing in new forms (host-loopback vs. bridge vs. mDNS vs. srflx). This is the 9th attempt.
**Recommendation: move the decode proof to an environment where the browser and SFU share a real
routable network** — a Linux CI runner with host networking, or a deployed staging host where the
gateway advertises a genuine reachable IP and a real STUN/TURN is available — rather than continue
peeling candidate-topology variants on the local VM.

## Next (carried) — pivot the proof environment, or fix the candidate topology properly

1. **Preferred — pivot:** run the decode on a **Linux host / CI runner with host networking** (no
   host↔VM↔bridge split), where the gateway advertises its real IP and the browser reaches it
   directly; or a deployed staging gateway with a public STUN/TURN.
2. **Or, if staying local:** add a **TURN relay** (coturn with `--external-ip` + credentials) so both
   peers get a relay candidate that always routes (the robust WebRTC answer to NAT/topology), and
   advertise the gateway's **bridge IP** (not `127.0.0.1`) via `MEDIA_ADVERTISE_IP`.
3. Re-run; flip only on `framesDecoded > 0`. The SFU engine, shared-socket demux, `.local` skip, and
   secure context are all proven; the remaining gap is a routable candidate pair.
