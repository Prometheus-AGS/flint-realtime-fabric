# Phase-29 decode result (p29-c003)

> Date: 2026-07-09. The decode proof re-run with **both** phase-28 blockers cleared — B2 (shared
> demuxing socket, c001) and B1 (STUN srflx + `.local` skip, c002). The run reached the furthest
> point in the whole sequence, but still produced **no decoded frame**. **`SFU_MODE=sovereign` stays
> gated off.** Evidence below, not a guess.

## Outcome: NOT proven — gate stays OFF

```
framesDecoded=0 bytes=0 reason=timeout ice=disconnected localCandidates=2 remoteCandidates=1
1 failed  ·  RUNNER_EXIT=1
```

## What c001 + c002 DID achieve (real, measurable progress)

**B2 (shared socket) is fixed — proven live:**

```
str0m::create_session{session_id=4561f86c…}: sovereign: session negotiated … bind=0.0.0.0:40000 advertised=…127.0.0.1 40000 typ host
str0m::create_session{session_id=6e27c39e…}: sovereign: session negotiated … bind=0.0.0.0:40000 advertised=…127.0.0.1 40000 typ host
```

**Two sessions negotiated on the ONE shared `0.0.0.0:40000` — no `EADDRINUSE`, no "unknown session".**
This is the two-peer room that was impossible in phase-28. The `frf_media_str0m::demux` module is doing
the routing (log target confirms the new code path).

**ICE advanced two states further than phase-28:**

| Metric | Phase-28 | Phase-29 | Meaning |
|---|---|---|---|
| gateway sessions | 1 (2nd `EADDRINUSE`) | **2, both negotiate** | B2 fixed |
| `remoteCandidates` (browser) | 0 | **1** | a gateway candidate now reaches the browser |
| `ice` state | `new` (never checked) | **`disconnected`** | ICE actually started + attempted connectivity |
| `.local` skip | crashed session (pre-c002) | **skipped, session alive** | B1 defensive skip working |

The `.local` candidates are still skipped (expected — that is B1's defensive skip, now explicit and
logged per session), and the session **survives** the skip. Everything c001/c002 set out to fix is
demonstrably working.

## The NEW blocker: ICE connectivity never completes (`disconnected`, no `Connected`)

```
frf_media_str0m::driver: sovereign: connection state session=4561f86c… state=Connecting
frf_media_str0m::driver: sovereign: connection state session=6e27c39e… state=Connecting
# … no `state=Connected` for either session; zero `MediaData`; zero fan-out.
```

Both sessions reach **`Connecting`** and stop; the browser ends at **`ice=disconnected`**. A candidate
pair formed (hence `remoteCandidates=1` and ICE leaving `new`), but it **never sustained connectivity
to DTLS/`Connected`**, so no RTP flows and nothing decodes.

### Root cause (evidence-led): the same-host loopback ↔ container-network candidate mismatch

- The gateway advertises **`127.0.0.1:40000`** — correct for a browser **on the host** reaching the
  container's published UDP port via host loopback.
- The browser's usable candidate is now a **STUN `srflx`** (coturn gave it one — that is why
  `remoteCandidates` went 0→1 and the `.local` host candidates are the only ones rejected). But the
  `srflx` reflects the browser's address **as coturn sees it on the Docker bridge network**, not the
  host-loopback address the gateway's `127.0.0.1` candidate can pair with.
- So the one pair that forms is between mismatched network views (host loopback vs. bridge srflx) and
  cannot complete a two-way connectivity check → `Connecting` → `disconnected`, never `Connected`.

This is a **candidate-topology / networking** issue in the same-host Docker harness, not a str0m
engine bug: the SFU negotiates, demuxes, and skips bad candidates correctly; the two peers simply
never share a routable address pair in this loopback-vs-bridge setup.

## Decision — honest gate held (operator-confirmed discipline)

`framesDecoded == 0`, no session reached `Connected`, zero `MediaData`. Per the discipline carried
since phase-16 and re-confirmed twice: **`SFU_MODE=sovereign` is NOT flipped.** `main.rs` keeps the
gate-off warning; SECURITY §6 keeps the media plane *composed but not proven*. The advance is real and
recorded — two sessions on one socket, ICE now attempting connectivity — and the remaining work is a
well-identified networking item.

## Next (carried)

1. **Fix the harness candidate topology** so the browser and the SFU share a routable pair on the same
   host. Options, in order of preference:
   - Run the **browser inside the Docker network** (a Playwright/Chromium container on the same
     compose network) so its host/srflx candidates and the gateway's are on one network — no
     loopback-vs-bridge split.
   - Or advertise a candidate the bridge-srflx can pair with (e.g. `host.docker.internal` / the
     host-gateway IP as `MEDIA_ADVERTISE_IP`) so both ends agree on a reachable address.
   - Or add host networking for the gateway in the sovereign override so `127.0.0.1` is literally
     shared.
2. Re-run the decode proof; **flip `SFU_MODE=sovereign` only on `framesDecoded > 0`.** Same gate.
