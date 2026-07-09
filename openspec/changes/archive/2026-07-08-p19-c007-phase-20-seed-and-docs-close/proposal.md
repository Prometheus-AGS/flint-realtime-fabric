# p19-c007 — phase-20 seed + phase-19 docs close

## Why

Phase-19 delivered its scoped work: admin-ui lint cleared (c001), ATProto outbound wired
into the gateway (c002), Dart deferral re-affirmed with a dated check (c003), the OIDC IdP
decision captured as ADR-004 (c004), LiveKit inbound-relay capability behind a trait seam
(c005), and the str0m live-UDP transport loop de-risked (c006). Two big items were
deliberately carried forward — the **full str0m sovereign SFU media loop** (c006 proved the
transport mechanics; the media fan-out is the remaining ~2–4wk) and **live cross-node
proofs** (LiveKit + str0m) that need a live server/browser. This change closes phase-19
honestly and seeds the dedicated **phase-20-sovereign-sfu-media-loop**.

## What Changes

Documentation + phase-seed only.

1. **Seed `phase-20-sovereign-sfu-media-loop`** — `goals.md` + skeleton `progress.json`,
   built from `SPIKE-FINDINGS.md` (post-c006) and the phase-19 carry-forwards: promote the
   `TransportLoop` into per-session async tasks, trickle ICE, DTLS/SRTP + RTP fan-out,
   per-room topology; plus the LiveKit live cross-node proof and the ADR-004 → OIDC build
   (once Accepted). **Does not flip any waypoint** — that is `/kbd-next-phase`'s job after
   `/kbd-reflect`; this only lays the seed directory.
2. **`docs/SECURITY.md` §6** — refresh each plane's status: ATProto outbound now wired
   (c002); str0m transport loop proven, media still deferred (c006); LiveKit inbound
   capability present, live proof integration-gated (c005); OIDC path = ADR-004 (c004);
   Dart re-affirmed (c003).
3. **`CHANGELOG.md`** — add the Phase 19 section (deliverables + what's deferred to
   phase-20).
4. **`docs/PHASE-19-SIGNOFF.md`** (new) — the honest close: gate-suite results, what
   functions end-to-end vs. re-affirmed deferred.

## Non-goals

- Advancing the waypoint or starting phase-20 (that follows `/kbd-reflect` +
  `/kbd-next-phase`).
- Any code change or flipping `SFU_MODE=sovereign` on.

## Impact

- Affected: `.kbd-orchestrator/phases/phase-20-sovereign-sfu-media-loop/` (seed),
  `docs/SECURITY.md`, `CHANGELOG.md`, `docs/PHASE-19-SIGNOFF.md` (new).
- Phase-19 closes with docs matching reality and phase-20 seeded from proven findings.
