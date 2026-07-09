# Assessment — phase-22-sovereign-sfu-npeer-pli

> Stage: Assess · 2026-07-08 · Backend: OpenSpec
> Grounds the seeded goals against the real `RoomRouter`, gateway sovereign branch, and str0m
> PLI API. This is the phase that *can* flip `SFU_MODE=sovereign` — so the assess job is to find
> the load-bearing external gate (a real browser) before planning, and re-scope goals the code
> already satisfies.

## Current state (code-grounded)

- **`RoomRouter::forward` is already N-peer general** — it iterates **all** other room members
  and pushes to each (`for peer in members.iter().filter(|&&s| s != from)`). Phase-21 only
  *proved* the 2-peer case. So G1 (N-peer fan-out) is **already implemented**.
- **str0m PLI API exists:** inbound `Event::KeyframeRequest(KeyframeRequest { mid, rid, kind })`;
  `Writer::request_keyframe(kind)` + `is_request_keyframe_possible(kind)`;
  `MediaData::is_keyframe()`. So the SFU forwards a receiver's keyframe request back to the
  sender — real, codeable.
- **Gateway sovereign branch is signaling-only:** `main.rs:269` (`SfuMode::Sovereign`) still
  builds `StrOmSignaler` and logs "no media will flow." `signal_service.rs` drives
  `MediaSignaler` (signaling), **not** `MediaTransport` — nothing routes `Offer`→`create_session`,
  `RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`, or relays `local_signals` back.
- **`StrOmTransport` (phase-21):** `create_session`/`join_room`/`add_remote_candidate`/
  `local_signals`/`connection_state`/`remove_session` + the `RoomRouter` forwarding path, all
  proven at the unit/integration layer. Ready to be *driven* by the gateway.

## Goal-by-goal grounding

### G1 — N-peer per-room fan-out · **ALREADY IMPLEMENTED — proof only** ✅

- `RoomRouter::forward` fans to all co-room peers today. The gap is a **test** (3+ peers, one
  sends, all others receive), not code. `#[cfg(test)] peer_count` already exists to assert
  membership; extend to an N-peer forwarding-delivery test.
- **Verdict:** a test change (+ maybe RID/simulcast handling *if* a real stream needs it —
  defer unless the browser proof surfaces it). Small.

### G2 — PLI / keyframe + renegotiation · **codeable; real-decode proof browser-gated** ◐

- Forward inbound `Event::KeyframeRequest` from a receiver to the room's sender(s) via the
  router (a new `ForwardedKeyframeRequest` alongside `ForwardedMedia`), and call
  `writer.request_keyframe` on the sender's `Rtc`. `is_request_keyframe_possible` guards it.
- Renegotiation (tracks/membership change) is the harder, browser-gated part.
- **Verdict:** the PLI-forwarding plumbing is codeable + unit-testable (request routes
  sender↔receiver); "a late peer decodes" is browser-gated.

### G3 — Gateway composition + end-to-end browser proof · **THE LOAD-BEARING GAP** ⚠️

- **G3.1 (codeable):** compose the gateway for `SFU_MODE=sovereign` — build **both**
  `StrOmSignaler` (signaling) and `StrOmTransport` (media), and extend the signal path
  (`signal_service.rs`) to drive `MediaTransport`: `Offer`→`create_session`,
  `RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`, relay `local_signals`
  outbound. This is real gateway integration (a `MediaTransport` field on the signal service /
  app state), but **all the port methods exist**.
- **G3.2 (browser-gated):** prove two **real browser** peers exchange *decoded* media through
  the sovereign gateway. This needs a real browser (WebRTC/DTLS/SRTP interop) — **the
  load-bearing external gate**, exactly like the IdP was for OIDC. It cannot be a unit test.
- **Verdict:** G3.1 codeable this phase; **G3.2 is browser-gated** — the operator must decide
  whether to stand up a browser E2E harness (Playwright + a WebRTC test page) or carry the
  proof. Nothing in G4 can proceed until G3.2 is decided.

### G4 — Flip `SFU_MODE=sovereign` · **strictly downstream of G3.2** ⚠️(gated)

- The flip is a small `main.rs` change (compose both ports live), but MUST be preceded by (a)
  G3.2's browser proof that media actually flows, and (b) covering the media path's boundary in
  `docs/SECURITY.md` §1–§5 (per-event RLS via Keto, tenant isolation, JWT verification on the
  media path). **Do not flip without the proof** — the six-phase discipline.
- **Verdict:** guardrail, last; flips **only** if G3.2 proves media flows, else re-affirm off.

### G5 — Live proofs (carried) · **external-infra gated** ◐

- LiveKit `realtime` data source (heavy dep + live server); admin-ui OIDC (ADR-004 + IdP).
  Both external-infra gated — re-affirm unless infra stood up.

## Gap summary & phase-shape recommendation

| Goal | Verdict | Codeable this phase? |
|------|---------|----------------------|
| **G1** N-peer fan-out | already implemented; **test only** | ✅ small (proof) |
| **G2** PLI forwarding | plumbing codeable; decode proof browser-gated | ◐ wire + unit-test |
| **G3.1** gateway composition | real integration; all port methods exist | ✅ codeable |
| **G3.2** browser E2E proof | needs a real browser | ⚠️ **decision: harness or carry** |
| **G4** flip the gate | small, last, **only if G3.2 proves media** | ✅ gated |
| **G5** live proofs | external-infra gated | ◐ re-affirm unless infra |

**Recommended phase shape (for `/kbd-plan`):**
1. **N-peer fan-out proof** (G1) — a test; the code already fans out.
2. **PLI forwarding** (G2) — route `KeyframeRequest` sender↔receiver via the router; unit-test.
3. **Gateway sovereign composition** (G3.1) — drive `StrOmTransport` from the signal path;
   compose both ports; still gated (media path present but unproven end-to-end).
4. **Browser E2E proof (G3.2) — operator decision:** stand up a Playwright + WebRTC test page
   to prove decoded media flows, OR carry it (re-affirm gated). This is the phase's crux.
5. **Flip `SFU_MODE=sovereign` (G4)** — only if G3.2 proves it; SECURITY §1–§5 boundary first.
   Else re-affirm off.
6. **G5** — integration-gated / infra-dependent.

Keeps the 16–21 discipline: re-scope what the code already does (G1), an honest browser gate
for the end-to-end proof, and `SFU_MODE=sovereign` flips **only** when media provably flows.

## Open questions for `/kbd-plan`

1. **Browser E2E harness (the crux):** stand up a Playwright + WebRTC test page this phase to
   prove decoded media end-to-end (which would let G4 flip the gate), or carry G3.2 and keep
   the gate off? (Assessment: this is the load-bearing decision; recommend deciding explicitly.)
2. **G1 scope:** confirm N-peer is a **test-only** goal (the router already fans out), unless a
   real stream needs RID/simulcast — defer that until the browser proof surfaces it.
3. **G4 boundary:** confirm the media-path security boundary (per-event Keto RLS + JWT on the
   media/signaling channel) is covered in SECURITY §1–§5 before any gate flip.
4. **G5 infra:** LiveKit server / IdP this phase, or carry again?
