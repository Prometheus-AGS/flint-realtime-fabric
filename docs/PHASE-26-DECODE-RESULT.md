# Phase-26 decoded-media proof — live run result (p26-c002)

> Date: 2026-07-09. The actual, un-fabricated outcome of the **authenticated** decode proof after
> the Dockerfile fix. This is the input that decides the `SFU_MODE=sovereign` flip (p26-c003).

## Result: ❌ NOT PROVEN — `framesDecoded > 0` not observed (but the MEDIA PATH is finally reached)

The run got **all the way to a real media exchange** — the entire stack booted, the gateway is
healthy, the participant is authorized, and Chromium ran the WebRTC harness. It failed on a
**Playwright 30s test timeout**: the receiver's decode promise never resolved. `RUNNER_EXIT=1`.
No decoded frame, so **G2 is not met** and c003 re-affirms the gate off — **but for the first time
the failure is a genuine media-transport symptom, not plumbing.**

## What actually happened — every layer now passes up to the media

```
[run-media-decode] minting an RS256 JWT + JWKS…                          ✓
[run-media-decode] JWKS reachable.                                       ✓
[run-media-decode] building the gateway image (if stale)…               ✓ (embeds admin-ui/dist)
[run-media-decode] bringing up the gateway + its deps…                  ✓ (iggy/keto/postgres/gateway)
[run-media-decode] gateway healthy.                                     ✓  ← booted, /healthz OK
[seed-media-view] OK (HTTP 201) — dev-integration-user may view room    ✓  ← ADR-007 view granted
[run-media-decode] running the decode harness with the authenticated JWT…
  media-decode.spec.ts › framesDecoded > 0 …
    Test timeout of 30000ms exceeded.                                   ✘  ← media never completed
RUNNER_EXIT=1
```

## The blocker arc across phases 24–26 (fully converged to the media path)

Every attempt cleared the prior blocker and surfaced the next; phase-26 cleared **five** in a row
and reached the media exchange itself:

| # | Blocker | Status |
|---|---------|--------|
| P24 | compose-merge no-JWT fallback (`undefined service keto`) | fixed |
| P25a | verifier RS256/JWKS vs flint-gate HS256 | fixed (self-mint RS256 + JWKS) |
| P25b | Dockerfile never built/copied `admin-ui/dist` (rust-embed) | **fixed (p26-c001, image builds)** |
| P26a | leaked `http.server` on the JWKS port → 404 | fixed (kill-stale + verify-our-keys) |
| P26b | production gateway refuses boot without `JWT_ISSUER` | fixed (compose `JWT_ISSUER` + mint `iss`) |
| P26c | `docker compose up --build` cold-rebuilt flint-gate every run → stall | fixed (gateway-only `up`, no flint-gate) |
| P26d | Keto v0.12 write API is `/admin/relation-tuples` (was `/relation-tuples` → 404) | fixed (seed path) |
| P26e | `getUserMedia` undefined on `about:blank` (insecure context) | fixed (goto gateway origin) |
| **P26f** | **the media exchange itself: offer/answer/ICE/DTLS/RTP → decode does not complete in 30s** | **the current, genuine media-path blocker** |

## Honest assessment

- **No decoded frame** — the proof does not pass. But the failure is now **at the media transport**:
  the receiver's `RTCPeerConnection`/`getStats().framesDecoded` never advanced within 30s, meaning
  the WebRTC negotiation (offer→answer→ICE→DTLS→RTP) did not complete to a decoded frame.
- **Everything around the media works** — build, boot, auth (RS256), the ADR-007 Keto `view` grant,
  and the browser harness all succeed. This is the deepest any run has reached by far.
- **This is real WebRTC debugging territory** (next-level, not a one-line fix): likely candidates —
  ICE candidate reachability over the `host.docker.internal:40000/udp` path (host↔container UDP,
  advertise-IP correctness), DTLS, or the harness's own sender↔receiver room-join/timing logic. It
  warrants a focused investigation, not a rushed patch.

## Consequence for the flip (p26-c003)

**`SFU_MODE=sovereign` stays gated OFF.** G2 unproven. c003 re-affirms gated with this concrete
media-transport blocker — no forced flip, no relaxed proof.

## To make the proof pass (next attempt / phase)

1. Instrument the harness + gateway: log ICE connection state, gathered/received candidates, and
   DTLS state on both peers; capture gateway media logs during the run.
2. Verify the sender↔receiver actually share the room and that the SFU relays the sender's RTP to
   the receiver (the `RoomRouter` fan-out over a real network, not in-process).
3. Confirm the advertised `host.docker.internal:40000/udp` candidate is reachable from the browser
   and the container binds/receives on it.
4. Re-run; observe `framesDecoded > 0`. Only then does the gate flip.
