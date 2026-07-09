# Phase-34 signoff — sovereign SFU decode: TURN relay

> Date: 2026-07-09. Phase-34 added a TURN relay on the bridge and confirmed str0m accepts `typ relay`,
> but **0 relay candidates reached the gateway** — coturn's relay address was the browser's own
> in-container loopback (the same bridge address-confusion in a TURN variant).
> **`SFU_MODE=sovereign` stays gated OFF** — and the decode proof is **escalated to CI / a real Linux
> host (Target B)**; no more local same-host variants.

## Gate decision: OFF (honest gate held) + CI escalation

No `framesDecoded > 0`; no `typ relay`; no `Connected`; no `MediaData`. Per the phase-16→33 discipline:
the gate does not flip until a real receiver observes a decoded frame. `main.rs` untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p34-c001 | coturn STUN-only → TURN relay (realm + lt-cred + env secret + external-ip); harness `iceServers` turn: + creds (both PCs) | none (harness); S1-clean |
| p34-c002 | Bridge+TURN decode run + honest gate decision + CI escalation | **held OFF** |

## Evidence

- ✅ **str0m accepts `typ relay`** (`str0m-0.21/src/sdp/parser.rs:232` → `CandidateKind::Relayed`) —
  the SFU-side of TURN is correct, no engine change.
- ✅ Sessions negotiate, `getUserMedia` works, ICE `checking` — the phase-32/34 baseline.
- ❌ **0 `typ relay` candidates reached the gateway** (`grep "typ relay"` = 0); only mDNS `.local`
  candidates, all skipped; `Connecting → Disconnected`; `framesDecoded=0`. coturn's
  `--external-ip=127.0.0.1` is the browser's own container loopback, so any relay candidate it
  advertised was unroutable in-network.

## The decision — escalate to CI, stop generating local variants

This was the 11th blocker and the **sixth candidate-topology form** (host-loopback / bridge / mDNS /
srflx / relay-external-ip) to fail on the same-host macOS + Colima-VM + Docker-bridge environment. The
media path, shared-socket demux, `.local` skip, secure context, and TURN acceptance are all **proven
correct in pieces**; the environment is the sole recurring blocker, and each local fix trades one
address confusion for another.

## Carried to next phase — CI / real Linux host (Target B)

1. **Containerize the proof rig** (JWKS mint/serve + Keto seed) so the whole decode runs as **one job
   on `ubuntu-latest`** (or a self-hosted Linux runner), where **host networking is native** — the
   browser and SFU share one real stack, host candidates pair with no bridge/loopback/mDNS confusion,
   TURN (already wired) is belt-and-suspenders, and the gateway advertises its real reachable IP.
2. Re-run there; **flip `SFU_MODE=sovereign` only on `framesDecoded > 0`.**

The Rust engine and every media-path layer are done. The residual is purely the proof environment —
and a real Linux host is the answer, not another local same-host variant.

## Verification

- Host `cargo test -p frf-media-str0m` (31), `cargo fmt --check` — green (no engine change).
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the CI escalation.
- **S1 (no committed secret):** the TURN credential is `${TURN_SECRET:?...}` (env-required, no
  default) in compose; the runner generates a random per-run secret.
