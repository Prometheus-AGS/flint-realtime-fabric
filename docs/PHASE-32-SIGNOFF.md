# Phase-32 signoff — sovereign SFU decode: HTTPS secure context

> Date: 2026-07-09. Phase-32 fixed the secure context (Caddy TLS sidecar) — `getUserMedia` now works,
> sessions negotiate, and ICE reaches `checking`, the furthest point in the whole 24→32 sequence. But
> `framesDecoded=0`: the browser produces only mDNS `.local` candidates (no usable STUN srflx), so no
> routable pair forms. **`SFU_MODE=sovereign` stays gated OFF** — and we recommend pivoting the proof
> to a real Linux/CI host.

## Gate decision: OFF (honest gate held) + environment-pivot recommended

No `framesDecoded > 0`, no session reached `Connected`, no `MediaData`. Per the phase-16→31
discipline: the gate does not flip until a real receiver observes a decoded frame.
`crates/frf-gateway/src/main.rs` untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p32-c001 | Caddy TLS sidecar → `https://caddy:8443` secure context; `--ignore-certificate-errors`; p31 unsafe flags removed | none (harness); secure context fixed |
| p32-c002 | Decode run over HTTPS + honest gate decision + environment-pivot recommendation | **held OFF** |

## Evidence — the furthest point yet

- ✅ **Secure context fixed:** `getUserMedia` succeeds (was `undefined` in p31); the sender acquires a
  track and offers.
- ✅ **Sessions negotiate + ICE reaches `checking`** (`remoteCandidates=1`) — up from p30
  `disconnected`/`new`. The shared-socket demux (p29 B2) + `.local` skip (p29 B1) hold live.
- ❌ **No routable candidate pair:** 0 STUN `srflx` candidates; 12 mDNS `.local` host candidates
  skipped; gateway advertises `127.0.0.1` (the browser's own loopback in-container). 0 `Connected`, 0
  `MediaData`. ICE stalls `checking` → `Disconnected`.

## The decision: pivot the proof environment

This was the **9th decode attempt** on the same-host Colima VM. The media-path *code* is proven
correct as far as any in-process test can show; the residual is a **routable candidate pair** the
local host↔VM↔bridge / mDNS / srflx / loopback topology keeps re-surfacing in new forms.

**Recommendation:** run the decode proof on a **Linux host / CI runner with host networking** (browser
+ SFU on one real network; gateway advertises a reachable IP; real STUN/TURN), or a deployed staging
gateway — rather than continue peeling candidate-topology variants locally. Alternatively (if staying
local) add a **TURN relay** (coturn `--external-ip` + credentials) and advertise the gateway's
**bridge IP** via `MEDIA_ADVERTISE_IP` so both peers always get a routing candidate.

## Verification

- Host `cargo test -p frf-media-str0m` (31), `cargo fmt --check`, admin-ui eslint — green (no engine
  change).
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the pivot
  recommendation.
