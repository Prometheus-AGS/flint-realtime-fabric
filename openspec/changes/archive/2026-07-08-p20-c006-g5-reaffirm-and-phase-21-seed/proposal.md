# p20-c006 — G5 re-affirm + phase-21 seed + phase-20 close

## Why

Phase-20 built the sovereign SFU media plane up to its scoped milestone: the `MediaTransport`
port (c002), an async per-session str0m engine (c003), trickle-ICE wiring (c004), and the
DTLS-connected milestone + crypto install (c005). Two things remain by design: the **full
RTP forwarding media loop** (split to phase-21 per ADR-005) and the **live cross-node /
OIDC proofs** (G5, no live infra this phase). This change closes phase-20 honestly and seeds
phase-21.

## What Changes

Documentation + phase-seed only.

1. **Seed `phase-21-sovereign-rtp-forwarding`** — `goals.md` + skeleton `progress.json`,
   from `SPIKE-FINDINGS.md` (post-c005) and the carry-forwards: RTP `MediaData` forwarding
   between peers (`writer(mid).write`), per-room fan-out + PLI, the offerer/peer role that
   unblocks the two-peer DTLS-connected + media proof, then flipping `SFU_MODE=sovereign`
   only once media flows; plus the carried G5.1 LiveKit live proof and G5.2 admin-ui OIDC.
   **No waypoint flip** — that's `/kbd-next-phase`'s job after `/kbd-reflect`.
2. **`docs/SECURITY.md` §6** — refresh the str0m Media row: async engine + trickle ICE +
   DTLS-connected milestone reached; **media still deferred (phase-21)**; `SFU_MODE=sovereign`
   still gated off. Re-affirm LiveKit inbound (G5.1) and admin-ui OIDC (G5.2) as carried.
3. **`CHANGELOG.md`** — add the Phase 20 section (deliverables + deferred-to-phase-21).
4. **`docs/PHASE-20-SIGNOFF.md`** (new) — gate results + functions-vs-deferred close note.

## Non-goals

- Advancing the waypoint / starting phase-21.
- Any code change or flipping `SFU_MODE=sovereign` on.

## Impact

- Affected: `.kbd-orchestrator/phases/phase-21-sovereign-rtp-forwarding/` (seed),
  `docs/SECURITY.md`, `CHANGELOG.md`, `docs/PHASE-20-SIGNOFF.md` (new).
- Phase-20 closes with docs matching reality and phase-21 seeded from the connected-milestone
  engine.
