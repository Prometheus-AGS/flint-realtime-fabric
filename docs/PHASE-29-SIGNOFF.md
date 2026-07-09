# Phase-29 signoff — sovereign SFU shared socket + STUN

> Date: 2026-07-09. Phase-29 cleared both phase-28 blockers (B2 shared demuxing socket, B1 STUN
> srflx) and got ICE attempting connectivity for the first time. **`SFU_MODE=sovereign` stays gated
> OFF** — the decode still yields no frame; the gap narrowed to one same-host networking item.

## Gate decision: OFF (honest gate held)

`framesDecoded == 0`; no session reached `Connected`; zero `MediaData`. Per the phase-16→28
discipline, re-confirmed by the operator twice: **the gate does not flip until a real receiver
observes `framesDecoded > 0`.** `crates/frf-gateway/src/main.rs` is untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p29-c001 | Shared demuxing `UdpSocket` (ADR-008) — `Rtc::accepts()` demux, single owning task; two sessions on one port, no `EADDRINUSE` | none (adapter); B2 fixed |
| p29-c002 | coturn STUN in compose + harness `iceServers` + explicit `.local` skip test | none (harness/adapter); B1 addressed |
| p29-c003 | Decode re-run + honest gate decision; `PHASE-29-DECODE-RESULT.md` | **held OFF** |

## Evidence

- **B2 fixed (live):** `session negotiated … bind=0.0.0.0:40000` for **two** sessions, no
  `EADDRINUSE`, `frf_media_str0m::demux` doing the routing.
- **ICE advanced:** browser `remoteCandidates` 0→1, `ice` `new`→`disconnected`; both sessions reach
  `Connecting`. The `.local` candidates are skipped and the session survives (B1 skip working).
- **New blocker:** neither session reaches `Connected`; no `MediaData`. The browser's bridge-network
  `srflx` and the gateway's `127.0.0.1` host candidate are mismatched network views (same-host
  loopback vs. Docker bridge), so the pair can't sustain connectivity.

## Carried to next phase

1. **Fix the harness candidate topology** so browser + SFU share a routable pair on one host:
   run the **browser inside the Docker network**, advertise a bridge-reachable `MEDIA_ADVERTISE_IP`
   (host-gateway), or give the gateway host networking.
2. Re-run the decode proof; flip only on `framesDecoded > 0`.
3. G4 carried (LiveKit x-node, admin-ui OIDC) — integration-gated.

## Verification

- `cargo check -p frf-media-str0m -p frf-gateway`, clippy pedantic, `cargo test -p frf-media-str0m`
  (31 tests incl. the shared-socket + `.local`-skip proofs), fmt, admin-ui eslint — green.
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the new blocker.
