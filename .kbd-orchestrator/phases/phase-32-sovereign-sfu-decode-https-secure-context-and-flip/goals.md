# Goals — phase-32-sovereign-sfu-decode-https-secure-context-and-flip

> Seeded from: phase-31 (reflection `summaryForNext`). Phase-31 removed the phase-30 build OOM (c001
> decouple — proven: the gateway image builds cleanly run alone) and cleared FOUR harness layers
> (pnpm-workspace mount, Playwright image v1.61, `DECODE_ONLY` webServer skip, secure-context flag
> attempt) so the decode now boots the full stack and STARTS the spec. But it stops at a fifth
> blocker: the in-network browser navigates to the **insecure** origin `http://gateway:8080`, so
> `navigator.mediaDevices` is undefined and `getUserMedia` throws before any offer. The
> `--unsafely-treat-insecure-origin-as-secure-origin` flags did NOT register `mediaDevices` in the
> headless Playwright container. See `docs/PHASE-31-DECODE-RESULT.md`. G1 (decouple) MET; G2 (decoded
> frame) NOT met. This is a harness secure-context detail — **upstream** of the media path, which is
> proven/in-place (candidate IP, shared socket, mDNS skip, STUN, in-network topology).

This phase gives the in-network browser a **genuine secure context** so `getUserMedia` works with NO
unsafe flags, runs the decode end-to-end, and — **only** on a real `framesDecoded > 0` — flips
`SFU_MODE=sovereign`. This is the run where the whole media path finally gets exercised end-to-end.

**Context (operator accounting, 2026-07-09):** this is the 9th decode-proof attempt (phases 24→32);
the gate has held OFF through all prior 8, every failure recorded honestly. The media *engine* was
done at phase-21; 22→31 have been peeling harness/environment blockers. If HTTPS reveals yet another
harness layer rather than a decoded frame, that is a strong signal to move the proof to a real CI
runner / different host rather than continue peeling on the unstable local Colima VM.

**PREREQUISITE (operational, via `!`):** the Colima daemon may be down (it crashed twice in p31).
`colima start` (or `restart`), then build the gateway image once (`docker compose … build gateway` —
c001's fail-fast enforces it). Keep concurrent VM load low.

**Discipline (carried 16→31):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (phase-29). Absence of a
defect is not presence of a proof (phase-30). Stop live-iterating when a blocker becomes flag-arcana
with no media signal (phase-31).

---

## G1 — Give the in-network browser a genuine secure context

Phase-31 evidence: `getUserMedia` throws (`navigator.mediaDevices` undefined) on the insecure
in-network origin `http://gateway:8080`; unsafe-origin flags did not register `mediaDevices` in the
headless container.

- **G1.1 (primary):** serve the gateway (or a thin TLS proxy in front of it) over **HTTPS** in the
  decode stack — a self-signed cert at e.g. `https://gateway:8443`, the browser launched with
  `--ignore-certificate-errors`. `https://` is a genuine secure context, so `navigator.mediaDevices`
  exists and `getUserMedia`/`getStats` work **with NO unsafe flags** — remove the
  `--unsafely-treat-insecure-origin-as-secure-origin` / `--disable-features` flags added in p31.
  Options: a TLS-terminating sidecar (nginx/caddy) on the compose network fronting `gateway:8080`, or
  the gateway serving TLS directly if it supports it. Harness/compose only — no `frf-*` engine change
  unless the gateway must terminate TLS itself (ADR if so).
- **G1.2 (alternative):** reach the gateway as a `*.localhost` origin (Chromium treats `*.localhost`
  as potentially-trustworthy → secure context) via a compose network alias `gateway.localhost` that
  the in-network browser resolves. Lighter than TLS if it resolves cleanly in-network.
- **G1.3** Point the harness `GATEWAY_URL`/`WS_URL` at the secure origin; keep the p31 in-network +
  DECODE_ONLY + STUN wiring. File-size ≤500; no library `unwrap`/`expect`.

**Exit:** the decode spec's browser has `navigator.mediaDevices` defined and `getUserMedia` succeeds
(a track is acquired); the run proceeds to WS signaling + `create_session` (gateway str0m logs show a
negotiated session), or the concrete next blocker is diagnosed + recorded.

## G2 — Observe a real decoded frame + flip

- **G2.1** Re-run `scripts/run-media-decode.sh`; observe `ice=connected`, `state=Connected`, inbound
  `MediaData`/fan-out, and `getStats().framesDecoded > 0`. Capture `docs/PHASE-32-DECODE-RESULT.md`
  (browser assertion + gateway str0m logs).
- **G2.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-32-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G2.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G3 — Carried live proofs (from phase-19…31)

- **G3.1** LiveKit cross-node inbound (`realtime` feature). **G3.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**).
- `SFU_MODE=sovereign` is enabled **only** if G2 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001…008 without a new finding.
- Net-new media features. TURN relay only if the topology genuinely needs a relay (STUN srflx already
  lands, p29).
- Chasing further Chromium unsafe-origin flag combinations — the durable fix is a real secure context
  (HTTPS / `*.localhost`), not more flags (phase-31 lesson).

## Starting point (from phase-31)

- Media path proven/in-place; prebuilt-image runner (no in-run build); in-network Playwright service +
  DECODE_ONLY + coturn STUN + repo-root mount + image v1.61.
- Blocker to clear: **secure context for `getUserMedia`** on the in-network origin (G1).
- Colima VM unstable under load; daemon may be down. Build once, keep load low.
