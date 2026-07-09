# p21-c005 — sovereign gate decision + phase-22 seed + phase-21 close

## Why

Phase-21 built the sovereign SFU media plane: session/driver split (c001), a real two-peer
DTLS-connected proof (c002), the RTP fan-out ADR (c003), and 1-to-1 RTP forwarding (c004).
This change closes the phase and decides the `SFU_MODE=sovereign` gate.

## The gate decision (honest)

**`SFU_MODE=sovereign` stays gated off.** Phase-21 wired and layer-proved 1-to-1 forwarding
(router fan-out unit-tested, driver write path, `StrOmTransport` wiring integration-tested)
and proved two peers reach DTLS-connected in-process. But the **end-to-end proof — two
connected peers exchange decoded media through the gateway** — is browser/negotiation-gated
(matching codecs on both peers + a real decoding peer), which phase-21 did not stand up. Per
the phase-16–20 discipline ("do not advertise a plane as shipped until it functions end-to-end
or is re-affirmed deferred"), the gate is **not** flipped: the engine is present and proven at
the unit/integration layer, but the live media path is re-affirmed gated until the end-to-end
proof lands (phase-22, which also brings N-peer fan-out + PLI that a real deployment needs).

## What Changes

Documentation + phase-seed only. No `SFU_MODE` flip.

1. **Seed `phase-22-sovereign-sfu-npeer-pli`** — N-peer per-room fan-out + PLI/keyframe +
   renegotiation on the c004 registry, the end-to-end browser media proof, the gateway
   composition (`StrOmSignaler` + `StrOmTransport`) + the actual `SFU_MODE=sovereign` flip
   once media flows, and the carried G5 live proofs (LiveKit, OIDC). **No waypoint flip.**
2. **`docs/SECURITY.md` §6** — str0m Media: 1-to-1 RTP forwarding wired + layer-proven; DTLS
   two-peer connected proven; **end-to-end media + N-peer/PLI deferred to phase-22**;
   `SFU_MODE=sovereign` still gated off.
3. **`CHANGELOG.md`** — Phase 21 section (deliverables + deferred-to-phase-22).
4. **`docs/PHASE-21-SIGNOFF.md`** (new) — gate results + functions-vs-deferred close note,
   including the honest note that c004 archived on a clippy BLOCK and was corrected to ALL PASS.

## Non-goals

- Flipping `SFU_MODE=sovereign` (browser-gated end-to-end proof not yet done).
- Advancing the waypoint / starting phase-22.

## Impact

- Affected: `.kbd-orchestrator/phases/phase-22-sovereign-sfu-npeer-pli/` (seed),
  `docs/SECURITY.md`, `CHANGELOG.md`, `docs/PHASE-21-SIGNOFF.md` (new).
- Phase-21 closes with docs matching reality; the sovereign gate stays honestly off.
