# Goals — phase-31-sovereign-sfu-decode-prebuilt-image-and-flip

> Seeded from: phase-30 (reflection `summaryForNext`). Phase-30 implemented the browser-in-Docker
> topology fix (the media-path engineering is now COMPLETE — every phase-24…29 defect fixed) but the
> decode **never ran**: the decode runner does an **in-run gateway image build** (heavy admin-ui
> Vite/`tsc` compile) that **crashed the Colima Linux-VM** (`rpc error: Unavailable … EOF`, daemon
> down). See `docs/PHASE-30-DECODE-RESULT.md`. G1 (topology) is implemented but UNVERIFIED
> (`ice=connected` never observed); G2 (decoded frame) did not happen. The residual is purely
> **environmental (build/run capacity)**, not a media-path defect.

This phase removes the environmental blocker so the decode that phases 24→30 built can finally run,
and — **only** on a real `framesDecoded > 0` — flips `SFU_MODE=sovereign`.

**PREREQUISITE:** the Colima daemon is down (VM crashed). `colima start` (ideally `--memory 8`+)
before any Docker work this phase.

**Discipline (carried 16→30):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (pipeline-enforce guards on
the pre-update snapshot — phase-29 lesson). Absence of a defect is not presence of a proof — G1 stays
UNVERIFIED until a live run observes `ice=connected` (phase-30 lesson).

---

## G1 — Decouple the gateway image build from the decode run (unblock the environment)

Phase-30 evidence: the runner's in-run `build gateway` (compiles the admin-ui inside the image) OOMs
the Colima VM when combined with the full stack + the Playwright image.

- **G1.1 (primary):** build the gateway image **once, out-of-band** (a standalone
  `docker build`/`docker compose build gateway` step, or CI), then change
  `scripts/run-media-decode.sh` to `up -d --no-build gateway` against the **pre-built** image — no
  in-run build. The runner should **fail fast with a clear message** if the image is absent
  (pointing at the out-of-band build step), never silently rebuild.
- **G1.2 (belt-and-suspenders):** document/raise the Colima VM memory (`colima start --memory 8`+) so
  even the pre-built stack + two browsers run comfortably. Restart the daemon first.
- **G1.3** No `frf-*` engine change (the engine is proven). Harness/compose/script + docs only.
  File-size ≤500.

**Exit:** `scripts/run-media-decode.sh` boots the full in-network stack (gateway + coturn +
playwright) against a pre-built gateway image **without** OOMing the VM, and the decode harness
actually executes (reaches the browser connect + gateway str0m logs).

## G2 — Observe a real decoded frame + flip

- **G2.1** Re-run the in-network decode; observe `ice=connected`, `state=Connected`, inbound
  `MediaData`/fan-out, and `getStats().framesDecoded > 0`. Capture `docs/PHASE-31-DECODE-RESULT.md`
  (browser assertion + gateway str0m logs).
- **G2.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-31-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G2.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G3 — Carried live proofs (from phase-19…30)

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
- Net-new media features; TURN relay (STUN srflx already lands). Any engine change requires a live
  finding once the decode actually runs.

## Starting point (from phase-30)

- Media-path complete (candidate IP, shared socket, mDNS skip, STUN, in-network topology all fixed).
- Harness plumbing fixed (repo-root mount, host-install reuse, Playwright image v1.61.0).
- Blocker to clear: **the in-run gateway image build OOMs the Colima VM** (G1). Daemon is DOWN —
  `colima start` first.
