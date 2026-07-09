# Reflection — phase-32-sovereign-sfu-decode-https-secure-context-and-flip

> Date: 2026-07-09. Phase goal: give the in-network browser a genuine secure context so `getUserMedia`
> works, run the decode end-to-end, and flip `SFU_MODE=sovereign` on `framesDecoded > 0`.
> **Outcome: G1 MET — the TLS sidecar fixed the secure context; `getUserMedia` works, sessions
> negotiate, and ICE reaches `checking` (the furthest point in the whole 24→32 arc). G2 NOT met:
> `framesDecoded=0` because the browser offers only mDNS `.local` candidates (no usable STUN srflx),
> so no routable pair forms. Gate held OFF; environment-pivot recommended.**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — secure context** | ✅ MET | Exit was "`navigator.mediaDevices` defined and `getUserMedia` succeeds; the run proceeds to WS signaling + `create_session`." The Caddy TLS sidecar (`https://caddy:8443` → `wss://`) gave the browser a genuine secure context; **`getUserMedia` succeeded** (the p31 blocker is gone, no unsafe flags), the sender offered, and the gateway logged multiple **negotiated sessions** with ICE reaching **`checking`**. Fully met — this is exactly the exit. |
| **G2 — decoded frame + flip** | ❌ NOT MET (gate correctly held) | `framesDecoded=0`; no session reached `Connected`; 0 `MediaData`. The browser produces **only mDNS `.local` host candidates** (str0m rejects them) and no STUN `srflx`; the gateway advertises `127.0.0.1` (the browser's own loopback in-container) — no routable pair, ICE stalls `checking` → `Disconnected`. `main.rs` untouched. |
| **G3 — carried proofs** | ◐ CARRIED | LiveKit x-node + admin-ui OIDC untouched; integration-gated. |

**Honest headline:** **1 of 2 goals MET (G1) — and it was a real breakthrough.** For the first time in
the whole 24→32 sequence the media exchange runs end-to-end: track acquired, offer sent, session
negotiated, ICE actively `checking`. But the objective (a decoded frame) still did not happen — the
residual is a genuine routable-candidate-pair problem, and it is the **9th** attempt on the same-host
VM. The needle moved substantially; it did not reach the goal.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p32-c001-tls-sidecar-secure-context | Caddy TLS sidecar → `https://caddy:8443` secure context; `--ignore-certificate-errors`; p31 unsafe flags removed | none (harness); secure context fixed |
| p32-c002-decode-run-and-flip | Decode over HTTPS + honest gate decision + environment-pivot recommendation | **held OFF** |

Both archived (`openspec/changes/archive/2026-07-09-p32-c00{1,2}-*`); `media-e2e` +
`sfu-mode-consistency` specs updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 2/2 |
| First-pass pass rate | 2/2 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Both changes passed R1–R5, S1, P1 on the first pass. 31/31 `frf-media-str0m` tests + admin-ui
eslint green on the host (no engine change).

## What actually moved (honest — the biggest single-phase jump)

- **`getUserMedia` works** — the phase-31 blocker is gone; the browser acquires a track.
- **The media exchange runs end-to-end for the first time:** offer → `create_session` → ICE
  `checking`. The SFU engine, shared-socket demux, and `.local` skip are all exercised by a real
  browser and behave correctly.
- **Still no decoded frame, no `Connected`.** The residual is a routable candidate pair — not the
  objective.

## Technical debt introduced

- **None net-new in shipped code** (harness/compose/TS only; no engine change). The Caddy sidecar is a
  clean, standard pattern.
- **The decode harness scaffolding is now large** (self-mint JWKS, Keto seed, coturn, in-network
  Playwright, DECODE_ONLY, Caddy TLS, secure-context handling). It is a full proof rig; on a pivot to
  CI it should be consolidated into a single reproducible compose profile.

## Lessons captured

1. **The layered peel finally reached the media layer.** Phases 24→31 peeled *harness/environment*
   (auth, Docker, embed, timing, socket, OOM, secure context). Phase-32's secure-context fix removed
   the last harness veil and exposed the **actual remaining media-path problem**: a routable candidate
   pair. That is progress of a different kind — we are now debugging WebRTC connectivity, not tooling.
2. **A self-hosted, same-host Docker VM is the wrong environment for a browser↔SFU media proof.** The
   host↔VM↔bridge split plus mDNS-hides-host-IPs plus loopback-is-the-browser means the two peers
   never trivially share a routable address. This has re-surfaced in four forms across phases 28–32.
   The durable answer is not another local variant — it is a **real network** (host networking / CI /
   staging + STUN/TURN). Recognizing *when the environment is the wrong tool* is itself the lesson.
3. **The honest gate held a seventh time — now against the most compelling near-miss.** "getUserMedia
   works, sessions negotiate, ICE is checking" is the closest the proof has ever looked. It still
   didn't flip: `framesDecoded == 0`, no `Connected`. Near ≠ done.
4. **Diagnostics compound.** Each phase's instrumentation (p27 ICE logs, p28 candidate detail, p29
   demux logs) made this phase's root cause readable at a glance (0 srflx / 12 `.local` skipped /
   `checking`→`Disconnected`). Investing in visibility early pays off many phases later.

## Recommended next phase

**`phase-33-sovereign-sfu-decode-real-network-proof`** — move the decode proof off the same-host VM to
an environment where the browser and SFU share a routable network, and add a TURN relay so a pair
always forms.

- **G1 (primary — the pivot):** run the decode on a **Linux host / CI runner with host networking**
  (browser + SFU on one real network stack; the gateway advertises its real reachable IP; no
  host↔VM↔bridge split). This is the environment the media proof actually needs. Consolidate the proof
  rig into one reproducible compose profile + a CI job.
- **G2 (belt-and-suspenders):** add a **TURN relay** — coturn with `--external-ip` + credentials, and
  the harness `iceServers` includes the `turn:` URL — so both peers always get a **relay candidate**
  that routes regardless of host/mDNS/srflx topology (the standard WebRTC answer to exactly this
  failure). Advertise the gateway's **bridge/real IP** via `MEDIA_ADVERTISE_IP`, not `127.0.0.1`.
- **G3:** re-run; observe `ice=connected` + `state=Connected` + `MediaData` + `framesDecoded > 0`.
  **Flip `SFU_MODE=sovereign` only on a genuine decoded frame.**
- **G4** carried (LiveKit x-node, admin-ui OIDC) — integration-gated.

**Discipline carried (16→32):** the gate flips only on decoded media proven against a live gateway;
else it stays off with fresh rationale. Update `progress.json` to N/N before any next-stage command
(phase-29). Absence of a defect is not presence of a proof (phase-30). Stop peeling a fragile local
environment when the *environment itself* is the blocker — pivot to a real one (phase-32).
