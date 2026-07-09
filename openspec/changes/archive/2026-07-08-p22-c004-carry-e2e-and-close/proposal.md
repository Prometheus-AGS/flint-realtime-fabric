# p22-c004 — carry the browser E2E proof + gate flip; close phase-22

## Why

Phase-22 delivered the codeable in-process work: N-peer fan-out proven (c001), PLI/keyframe
forwarding (c002), and the gateway sovereign composition (c003 — the media plane is now
drivable from the signal path). The **end-to-end browser proof** (two connected peers exchange
decoded media, G3.2) and the **`SFU_MODE=sovereign` flip** (G4) were carried by operator
decision — they need a real browser WebRTC/DTLS/SRTP harness. This change closes the phase and
seeds a follow-on.

## The gate decision (honest)

**`SFU_MODE=sovereign` stays gated off.** The media plane is fully composed and layer-proven
(N-peer fan-out, PLI routing, gateway driving `MediaTransport` from the signal path), but no
real browser peer has exchanged decoded media through it — the E2E proof is browser-gated and
was not stood up this phase. Per the phase-16–21 discipline, the gate is not flipped until
media provably flows end-to-end; hosted (LiveKit) remains the production media path.

## What Changes

Documentation + phase-seed only. No `SFU_MODE` flip.

1. **Seed `phase-23-sovereign-sfu-e2e-and-gate`** — the browser E2E harness (Playwright + a
   WebRTC test page), the decoded-media proof, the `SFU_MODE=sovereign` flip (with the media
   path's boundary covered in SECURITY §1–§5: per-event Keto RLS, tenant isolation, JWT), and
   the carried G5 live proofs. **No waypoint flip.**
2. **`docs/SECURITY.md` §6** — str0m Media: N-peer fan-out + PLI forwarding + gateway
   composition present + layer-proven; **end-to-end browser proof + gate flip deferred to
   phase-23**; `SFU_MODE=sovereign` still gated off.
3. **`CHANGELOG.md`** — Phase 22 section (deliverables + deferred-to-phase-23).
4. **`docs/PHASE-22-SIGNOFF.md`** (new) — gate results + functions-vs-deferred close note,
   including the honest c003 note (clippy `doc_markdown` BLOCK caught before archive + fixed to
   ALL PASS — the phase-21 lesson applied).

## Non-goals

- Flipping `SFU_MODE=sovereign` (browser E2E proof not done).
- Advancing the waypoint / starting phase-23.

## Impact

- Affected: `.kbd-orchestrator/phases/phase-23-sovereign-sfu-e2e-and-gate/` (seed),
  `docs/SECURITY.md`, `CHANGELOG.md`, `docs/PHASE-22-SIGNOFF.md` (new).
- Phase-22 closes with docs matching reality; the sovereign gate stays honestly off.
