# Phase-28 media-transport diagnosis (p28-c002)

> Date: 2026-07-09. With the diagnostics surfaced (p28-c001), the decode run pinpointed the exact
> media-path defect. Evidence-led, not guessed.

## The evidence (now visible)

**Browser (harness assertion line):**
```
framesDecoded=0 bytes=0 reason=timeout ice=new localCandidates=2 remoteCandidates=0
```
The browser gathered its own candidates (localCandidates=2) but received **zero from the gateway**
(remoteCandidates=0), and ICE never left **`new`**.

**Gateway (`/tmp/p28-gateway.log`, str0m lifecycle):**
```
frf_gateway::media_bridge: sovereign: create_session failed
  error=… host candidate: ICE bad candidate: invalid ip 0.0.0.0
frf_gateway::media_bridge: sovereign: add_remote_candidate failed error=… unknown session
```
`create_session` fails at candidate construction; with no session, every subsequent
`add_remote_candidate` (the browser's trickle) hits "unknown session".

## Root cause

`negotiate` builds the SFU's ICE host candidate from `MediaConfig.advertise_ip` when set, else the
bound `local_addr`. But:

- `MediaConfig.advertise_ip` is typed `Option<IpAddr>`, parsed via `MEDIA_ADVERTISE_IP.parse().ok()`
  (`config.rs`).
- Compose sets `MEDIA_ADVERTISE_IP="host.docker.internal"` — a **hostname**, not an IP. `.parse::<IpAddr>()`
  **fails** → `advertise_ip = None`.
- So `negotiate` falls back to `local_addr` = **`0.0.0.0:40000`** (the bind address).
- `str0m::Candidate::host(0.0.0.0)` rejects the unspecified address → **`ICE bad candidate: invalid
  ip 0.0.0.0`** → `create_session` errors → no session → no gateway candidate reaches the browser →
  `remoteCandidates=0` → ICE stuck at `new` → 30s timeout, no decode.

This is neither a fan-out bug nor a DTLS bug — it is an **ICE candidate-IP configuration bug**: a
hostname was silently dropped and the code advertised the unroutable bind address.

## The fix (operator decision: resolve hostname → IP in code)

Change `MediaConfig.advertise_ip` from `Option<IpAddr>` to an optional **host-or-IP string**, and
resolve it to an `IpAddr` at negotiate time (accept a literal IP as-is; DNS-resolve a hostname). For
the same-host run, `host.docker.internal` resolves inside the container to the host-gateway IP, which
the browser on the host reaches because UDP `40000` is published to the host. A resolution failure is
a hard error (no silent fallback to `0.0.0.0`).

## Verification

After the fix, the gateway log should show `session negotiated (… advertised=<resolved IP>)` and
`connection state Connected`, and the browser should report `remoteCandidates>0` and `ice=connected`
— the prerequisites for `framesDecoded > 0` (p28-c003).
