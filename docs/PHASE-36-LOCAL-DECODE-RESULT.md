# Phase 36 — Local Decode Result

> `p36-c004-local-decode-run-and-flip`, run 2026-09-08 on the operator's macOS host.
> Supersedes the CI-dependent half of `p36-c002`, which the local-testing-only policy
> made unsatisfiable.
>
> **Gate decision: `SFU_MODE=sovereign` FLIPPED ON.** The decode proof passes. See §5.
>
> **Reading order matters.** §2-§4 describe a FAILED run (`ice=new`, no media). That run's
> failure had a fixable harness cause; §5a describes the fix and the PASSING run that
> followed. Both are kept because the sequence is the finding — two harness defects masked
> a working media path three separate times.

## 1. Topology

| Item | Value |
|---|---|
| Compose files | `compose.yml` + `compose.sovereign.yml` (**bridge**, not host-net) |
| Container runtime | **OrbStack** (not Docker Desktop — see §6) |
| Colima | not running, so the p33-c001 `HOST_NET=1` path was unavailable |
| Gateway image | `flint-realtime-fabric-gateway`, `sha256:968132ea6a86…`, built locally (`build_exit=0`) |
| Playwright | image `mcr.microsoft.com/playwright:v1.61.0`, CLI `@playwright/test` **1.61.0** (matched) |

The phase-30 image-build blocker did **not** reproduce here: that was a Colima VM OOM
during the admin-UI compile, and this host had enough memory.

## 2. Result

```
framesDecoded=0  bytes=0  reason=timeout  ice=new  localCandidates=1  remoteCandidates=1
```

Consistent across the initial attempt and both retries. `decode_exit=1`.

**Classification: (c) — ICE never connects. An ENVIRONMENT result, not a media-path result.**

Per this change's T4, outcome (c) *"must not be reported as"* a media-path result. It is
not being. Nothing here says the SFU cannot relay media; it says the two endpoints never
formed a candidate pair on this host's network topology.

### This is NOT the same failure as CI run 29112243615

Conflating them would be the easy mistake, and it would be wrong:

| Signal | run 29112243615 (CI) | this run (local) |
|---|---|---|
| `ice` | **connected** | **new** |
| `bytes` | ≈1,800,000 | **0** |
| `framesDecoded` | 0 | 0 |
| PLI / keyframe | **zero** | **fires** (§4) |
| `room=` | **empty** | **`"e2e-decode-room"`** |

CI got a connected transport carrying ~1.8 MB that decoded to nothing — a genuine
media-path symptom. This run never got a transport at all. **The `framesDecoded=0` is the
same number for two entirely different reasons**, and only the CI one is evidence about
the media path.

## 3. Root cause of the ICE failure: an IPv6 candidate

From `docs/evidence/p36-c004-gateway-capture.log`:

```
sovereign: session negotiated (offer accepted, host candidate advertised)
  bind=0.0.0.0:40000
  advertised=candidate:ffffff7e4f41079ba174c8fa 1 udp 2130706431 fd07:b51a:cc66:d002::6 40000 typ host
```

`compose.sovereign.yml` sets `MEDIA_ADVERTISE_IP: "gateway"`, and str0m resolves that
hostname at negotiate time via `to_socket_addrs()`. Under OrbStack that name resolves to an
**IPv6 ULA** (`fd07:…`), which the Playwright container did not pair with. The connection
went `Connecting → Disconnected` on every session, and coturn confirms it from the other
side — TURN allocations succeed but carry no peer traffic:

```
session …: incoming packet ALLOCATE processed, success
session …: peer usage: rp=0, rb=0, sp=0, sb=0
session …: closed (2nd stage), reason: allocation timeout
```

The p36-c001 comment in `compose.sovereign.yml` reasoned that the service name resolves to
"this container's own Compose-bridge IP", which the Playwright container reaches directly.
That reasoning holds on the CI Linux bridge. It does **not** hold on OrbStack, where the
name resolves IPv6-first.

## 4. T5 — the ADR-009 child's PLI/room work is CONFIRMED at runtime

This run produced the first populated gateway capture in the phase (25,638 bytes), and it
answers the question run 29112243615 could not:

```
sovereign: room joined → proactive PLI to co-room senders  room="e2e-decode-room"
sovereign: keyframe request applied → PLI to sender  mid=Mid(0)  kind=Pli
```

Both defects the child fixed are demonstrably gone:

| Defect (run 29112243615) | Now |
|---|---|
| zero PLI / FIR / keyframe events | **PLI fires**, repeatedly, per session |
| `room=` empty throughout | **`room="e2e-decode-room"`** |

This is a genuine upgrade over the unit-level evidence recorded in T5: the PLI path is now
observed on a live gateway, not merely unit-tested.

**One thing this does not prove.** The PLI fires while ICE is disconnected, so it is emitted
into a transport that never carried media. That the *request* is generated and routed is
confirmed; that it produces a keyframe at the sender is still unobserved. Do not upgrade
this into "the PLI works end-to-end".

## 5. Gate decision — **FLIPPED ON**

### 5a. The third fix, and the passing run

§3 identified the ICE failure as an IPv6 host candidate. The fix follows from it directly:
`MEDIA_ADVERTISE_IP` is now overridable in `compose.sovereign.yml` (still defaulting to
`gateway`, so CI is unchanged), and `run-media-decode.sh` resolves the gateway container's
**IPv4** address after boot and recreates the gateway with it.

Confirmed the ambiguity first, in-network:

```
$ getent ahosts gateway
192.168.117.5            STREAM gateway     <- v4
fd07:b51a:cc66:d002::5   STREAM gateway     <- v6, what str0m was picking
```

With the IPv4 pinned, the decode **passes**:

```
[run-media-decode] pinning MEDIA_ADVERTISE_IP=192.168.117.5 (IPv4; avoids the IPv6 host candidate)
[run-media-decode] gateway healthy.
  1 passed (2.3s)
[run-media-decode] decode proof PASSED — framesDecoded > 0 observed (authenticated path).
decode_exit=0
```

**Reproduced twice**, the second time on a different container IP (`192.168.117.6`),
confirming the resolution is dynamic rather than a lucky fixed address.

**Classification: (a) — `framesDecoded > 0`. The media path works.**

### 5b. Verified genuine, not a skipped or vacuous pass

Given how many false-greens this phase has produced, the pass was checked rather than
trusted:

| Check | Result |
|---|---|
| Was the test skipped? | No — Playwright reported `1 passed`; a skip reports as skipped |
| Is the skip gate active? | No — `test.skip` fires only on `SKIP_INTEGRATION=true` or unset `GATEWAY_URL`; the runner sets `SKIP_INTEGRATION=false` |
| What does the assertion assert? | `result.decoded`, set at `decode-probe.ts:123` **only** when `frames > 0` from live `inbound-rtp` stats |
| Is the exit code trustworthy? | Captured from the runner directly, not through a pipe |
| Same assertion as the failures? | Yes — it printed `framesDecoded=0 … ice=new` twice before the fix |

### 5c. The flip

`SFU_MODE=sovereign` in `crates/frf-gateway/src/main.rs` no longer emits a warning. The
`tracing::warn!` *"end-to-end media is NOT yet proven (browser proof deferred)"* is replaced
by a `tracing::info!` recording the proof and pointing at the topology caveat. Clippy passes
(`-D warnings -W clippy::pedantic`, exit 0).

**What the proof does and does not cover.** It was a LOCAL run on one Compose bridge, both
peers inside the network, coturn available. It shows the relay decodes real media. It is
**not** a multi-host, NAT-traversal, or scale result, and the advertised-candidate config is
topology-sensitive — which is exactly what §3 demonstrates. Operators must verify
`MEDIA_ADVERTISE_IP` is peer-reachable for their own topology; the new log line says so.

Gate discipline from phases 16–36 is intact: the gate stayed shut through every run that
did not produce `framesDecoded > 0`, and opened on the run that did.

## 6. Harness defects found and fixed

Three real defects in the **local** path, all previously masked because CI supplied what
the local runner did not.

### 6a. `FLINT_GATE_JWT_SECRET` never exported (fixed)

`run-media-decode.sh` exported `TURN_SECRET` but not this, so compose aborted at
*interpolation* — before any container started. CI supplied it job-wide. Now defaulted in
the runner.

### 6b. `GATEWAY_URL` used `localhost`, which is IPv6-first (fixed)

The cause of the earlier "gateway never became healthy" abort. Verified directly, same
container, same moment:

| URL | Result |
|---|---|
| `http://[::1]:28080/healthz` | connection reset |
| `http://127.0.0.1:28080/healthz` | **HTTP 200** |
| in-container `http://localhost:8080/healthz` | **HTTP 200** |

`localhost` resolves `::1` first on macOS, and OrbStack's IPv6 forward resets while its
IPv4 forward serves normally. **A completely healthy gateway looked dead**, and the run
aborted before the browsers ever launched. Fixed by defaulting `GATEWAY_URL` to
`127.0.0.1`.

The JWKS polls at `:120`/`:126` also use `localhost`, but they target a host process rather
than a container port-forward, so they are unaffected and were left alone.

### 6c. No `node_modules` on the host (fixed by running the install)

`compose.sovereign.yml` mounts the repo and relies on *"the host already has a complete
install"*. CI installed as a job step; this host never had. Resolved with
`pnpm install --frozen-lockfile` (exit 0), which also produced the exact
`@playwright/test` 1.61.0 the image pins.

### 6d. The c001 log capture works — it had simply never been reached

Recorded because I twice wrote that the capture was broken. It was not. `/tmp/gateway-capture.log`
came out empty in CI and absent locally because **the run died before the capture line**.
Once the run reached the Playwright stage, the capture produced 25,638 bytes on the first
try. The `cp` ordering at `:184` was correct all along.

Both logs are preserved at `docs/evidence/p36-c004-*.log` rather than `/tmp`, so this
evidence survives the next run's teardown.

## 7. What this run did and did not settle

**Settled:**
- The sovereign stack builds, boots, negotiates, and runs the decode harness on this host.
- The ADR-009 PLI + room-registration fixes work on a live gateway (§4).
- Three local-path harness defects, all fixed in-repo (§6).

- **The sovereign SFU relays decodable media** (§5). Phase 36's central question is
  answered, and the gate is open.
- A fourth harness defect: the IPv6 advertised candidate (§3), fixed in §5a.

**Not settled:**
- Multi-host / NAT-traversal / scale behaviour. The proof is single-bridge and local.
- Whether the PLI produces a keyframe *at the sender*. PLI generation and routing are
  confirmed (§4); the sender-side response is still unobserved.

## 8. The through-line

Every one of the four defects found here (§6a, §6b, §6c, §3) was in the **harness or the
local path**, not the SFU. Three of them made a working media path look broken:

- `FLINT_GATE_JWT_SECRET` — aborted before any container started
- `localhost` → IPv6 — a healthy gateway reported as "never became healthy"
- missing `node_modules` — aborted at the Playwright exec
- `MEDIA_ADVERTISE_IP` → IPv6 — ICE stranded in `new`

Phases 28–36 accumulated evidence read as media-path trouble. At least the local portion of
it was the harness. **Two independent IPv6-before-IPv4 resolutions** (§6b, §3) were the
single most expensive root cause — worth checking anywhere else the harness resolves a name
that must yield a peer-reachable address.

The CI result (run 29112243615: `ice=connected`, ~1.8 MB, `framesDecoded=0`) is a genuinely
different failure and is **not** explained by any of this. It was on a Linux bridge where
these defects do not apply. Whether the ADR-009 PLI/room fixes resolve it is untested — it
would need a CI run, which policy forbids, so it stays open as a known difference rather
than a claimed fix.
