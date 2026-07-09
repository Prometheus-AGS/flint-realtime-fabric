# Reflection — phase-26-gateway-docker-adminui-embed-and-decode

> Generated 2026-07-09. Fixed the gateway image build and drove the authenticated decode run all
> the way to the media exchange for the first time. `SFU_MODE=sovereign` re-affirmed off — the
> WebRTC decode does not complete yet, but the frontier moved from plumbing to the media transport.

## Delta — movement against phase goals

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — fix Dockerfile admin-ui embed** | ✅ MET | Node 24 stage builds the admin UI (pnpm workspace + SDK siblings; `frf-wasm` stubbed) → `COPY admin-ui/dist` before `cargo build` → **gateway image builds** (`rust-embed` resolves). p26-c001. |
| **G2 — real decoded frame** | ❌ NOT PROVEN | Authenticated run reached: build → boot → **gateway healthy** → **Keto `view` granted (201)** → **browser harness runs** → WebRTC decode **timed out (30s)**; no `framesDecoded` (`PHASE-26-DECODE-RESULT.md`). p26-c002. |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet → no flip; `main.rs` warning intact; production defaults hosted. |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC. |

**1/4 MET (G1); G2/G3 did not close — but the run reached the media exchange for the first time.**

## Root cause — why G2/G3 did not close

The gateway image build was fixed (G1), so the run finally booted the full stack, authenticated,
authorized, and ran the browser harness. It then hit the **media transport itself**: the receiver's
`RTCPeerConnection`/`getStats().framesDecoded` never advanced within 30s — offer/answer/ICE/DTLS/RTP
did not complete to a decoded frame. This is genuine WebRTC (ICE candidate reachability over the
`host.docker.internal:40000/udp` path, DTLS, or the SFU's real-network RTP fan-out), not plumbing —
warranting a focused investigation, not a rushed patch.

## The blocker arc — converged to the media transport

Eleven phases of honest, convergent failures; **phase-26 cleared eight blockers in sequence** and
reached the media exchange:

`P24 compose-merge → P25a RS256/HS256 verifier → P25b Dockerfile embed → P26 { JWKS-port leak →
JWT_ISSUER boot → flint-gate --build stall → Keto /admin write path → getUserMedia secure-context }
→ **the WebRTC decode does not complete (media transport)**.`

## Delivered changes (3/3 archived)

1. **c001** — Dockerfile Node build stage + `COPY admin-ui/dist` → the gateway image builds (two
   build sub-issues iterated: nonexistent root `package.json`; SDK siblings need building first).
2. **c002** — executed the proof, hardened runner+harness through five concrete blockers to reach
   the media path, recorded the decode timeout honestly.
3. **c003** — re-affirm gate off + SECURITY §6 + CHANGELOG + PHASE-26-SIGNOFF.

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes | 3/3 archived |
| Final QA verdict | 3/3 ALL PASS |
| Process errors caught by a gate | 1 (c002 change dir never seeded — `verify` flagged "not found") |
| Release gate at close | fmt ✅ · clippy --workspace ✅ · check ✅ · str0m 27 · gateway 39 |

### Notable

- **c001 iterated on the real Docker build** — each failure (root `package.json`, unbuilt SDK
  siblings) was diagnosed from actual output and fixed, ending in a clean `Built` image.
- **The verify gate caught a genuine process slip:** I started c002 with `begin-task` but never
  seeded its openspec change dir; `verify: FAIL / not found` surfaced it, and I seeded it
  retroactively + archived cleanly rather than skip. The gate did exactly what it exists for.

## Technical debt introduced

- **None structural.** All the fixes are to the build/harness (Dockerfile, scripts, spec) — real,
  reproducible improvements. The un-observed decode is a documented, gated deferral with a precise
  media-transport next step.

## Lessons captured

1. **Iterate on the real artifact.** Both c001 (Docker build) and c002 (live run) only surfaced
   their true blockers by *running* — each attempt cleared one and exposed the next. Reasoning
   alone would have found none of them.
2. **The verify gate is not ceremony.** It caught a missing openspec seed that would otherwise have
   left a change half-recorded. Seed the change dir at the start of every `/kbd-apply`, and trust
   `verify` to catch it when you don't.
3. **Clean up shared host state between runs.** A leaked `http.server` and orphaned compose stacks
   from prior runs silently broke later ones (404, port conflicts). Harness runners must be
   idempotent — kill stale servers/stacks, pick free ports, verify *their own* output.
4. **`--build` on every compose up is a trap.** It cold-rebuilt an unrelated sibling-repo service
   (flint-gate) each run and stalled. Build the one image you changed; `up` only the services you
   need.

## Recommended next phase

**`phase-27-sovereign-sfu-media-transport-debug`** — the focused WebRTC investigation:

- Instrument both browser peers: log `iceConnectionState`/`connectionState`, gathered + received
  ICE candidates, and DTLS state; add a short timeout-with-diagnostics to the probe so a failure
  reports *where* it stalled (no candidates? no track? ICE stuck at `checking`?).
- Capture gateway media logs during the run; confirm the sender's offer is negotiated, the
  `RoomRouter` places sender+receiver in the same room, and it forwards the sender's RTP to the
  receiver over the real UDP path.
- Verify the advertised `host.docker.internal:40000/udp` candidate is reachable from the host
  browser and the container binds/receives on it (the p24 `MediaConfig` advertise-IP in practice).
- Re-run; observe `framesDecoded > 0`. **Only then** flip `SFU_MODE=sovereign` + SECURITY §6 + CHANGELOG.
- Fold in G4 (LiveKit `realtime`; admin-ui OIDC).

If the media-transport issue proves deep (e.g. str0m ICE behavior over mapped UDP needs rework),
split the finding into its own change and keep the gate off — do not force it.
