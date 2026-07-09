# Phase-34 decode result (p34-c002)

> Date: 2026-07-09. TURN was added to the bridge stack (c001) and str0m accepts `typ relay`
> (confirmed). But the run shows **0 `typ relay` candidates reach the gateway** — the browser still
> offers only mDNS `.local`, ICE stays `checking → Disconnected`, `framesDecoded=0`.
> **`SFU_MODE=sovereign` stays gated OFF.** This is the 11th blocker and the same bridge-address
> confusion in a new form — the recorded escalation point: **pivot the proof to CI / a real Linux host
> (Target B); stop generating local variants.**

## Outcome: NOT proven — gate stays OFF

```
framesDecoded=0 bytes=0 reason=timeout ice=checking localCandidates=5 remoteCandidates=1
1 failed  ·  RUNNER_EXIT=1
```

## What the run showed

- ✅ **Sessions negotiate + the shared-socket demux routes** (multiple sessions on `0.0.0.0:40000`, no
  `EADDRINUSE`); `getUserMedia` works; ICE reaches `checking` — the phase-32/34 baseline holds.
- ❌ **No relay candidate reaches the gateway.** `grep "typ relay"` on the gateway log = **0**. The
  gateway sees only mDNS `<uuid>.local typ host` candidates (all skipped), then
  `Connecting → Disconnected`. The TURN relay path did not produce a usable candidate the SFU could
  pair.

## Root cause (evidence-led): coturn's relay candidate address is wrong for the in-network browser

The TURN infrastructure is correct in principle (str0m accepts `typ relay`; `iceServers` wired with
credentials; coturn started). The failure is **the relay candidate address**: coturn ran with
`--external-ip=127.0.0.1` (the default). A TURN relay hands the client a **relay candidate at the
`external-ip`** — but `127.0.0.1` inside the browser's container is the **browser itself**, not
coturn. So even if coturn allocated a relay, the candidate it advertised is unroutable for the
in-network browser (and no `typ relay` reached the gateway). This is the **same bridge
loopback-vs-container address confusion** that defeated phases 28–33 — now in a TURN variant.

Fixing `--external-ip` to coturn's **bridge IP** might advance it one step — but that is exactly the
kind of local same-host-VM address variant the phase-32/33 conclusions said to **stop generating**.

## Decision — honest gate held + ESCALATE to CI (Target B)

`framesDecoded == 0`; no `typ relay`; no `Connected`; no `MediaData`. **`SFU_MODE=sovereign` is NOT
flipped.** `main.rs` untouched; SECURITY §6 keeps the plane *composed but not proven*.

**This is the escalation the plan drew** ("if TURN-on-bridge fails, pivot to CI (Target B) — no more
local variants"). Across phases 28→34 the media path, shared-socket demux, `.local` skip, secure
context, and TURN acceptance are all **proven correct**; the *only* recurring failure is the
**same-host Docker candidate-address topology** (host-loopback vs. bridge vs. mDNS vs. srflx vs. relay
external-ip). It has re-surfaced in six forms. The environment — a macOS + Colima VM + Docker bridge —
is fundamentally unsuited to a browser↔SFU media proof, and each local fix trades one address
confusion for another.

## Recommended next — CI / real Linux host (Target B), the durable home

1. **Containerize the proof rig** (JWKS mint/serve + Keto seed) so the whole decode runs as **one job
   on `ubuntu-latest`** (or a self-hosted Linux runner), where **host networking is native** — the
   browser and SFU share one real network stack, host candidates pair with no bridge/loopback/mDNS
   confusion, and TURN (already wired) is a belt-and-suspenders, not a workaround.
2. On that runner, the gateway advertises its **real reachable IP** (`MEDIA_ADVERTISE_IP`), and the
   whole 28→34 media stack — proven correct in pieces — finally runs end-to-end.
3. Re-run there; **flip `SFU_MODE=sovereign` only on `framesDecoded > 0`.**

The Rust engine, shared-socket demux, `.local` skip, secure context, and str0m relay acceptance are
all done. The residual is purely **the proof environment** — and the answer is a real Linux host, not
another local same-host variant.
