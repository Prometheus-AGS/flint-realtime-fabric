# Reflection — phase-20-sovereign-sfu-media-loop

> Generated: 2026-07-08
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **6 / 6 DONE**, all archived. Seeded from the phase-19 reflection.

## Summary

Phase-20 built the sovereign SFU media plane from the phase-19 transport-loop proof up to the
**DTLS-connected milestone** — deliberately *not* to media forwarding, which was split into
phase-21 up front. The assessment's decisive finding was that the seed goal "replace the
channel-only signaler" hid an architecture question: the `MediaSignaler` port is
signaling-only, so the media engine had no home. That became an ADR-first change (ADR-005 →
new `MediaTransport` port), exactly the discipline ADR-004 set in phase-19. The engine was
then built layer by layer — port trait, async per-session str0m loop, trickle ICE,
DTLS-connected — each gate-passable on its own, with the browser/peer-gated proofs honestly
`#[ignore]`d rather than faked. `SFU_MODE=sovereign` stays gated off: the phase reached
*connected*, not *media forwarded*.

## Goal Achievement (against the operator-scoped plan)

| Goal | Intent | Delivered | Verdict | Evidence |
|------|--------|-----------|---------|----------|
| **G1 arch** | resolve the media-plane home | ADR-005: new `MediaTransport` port | **MET** | c001; recommend + rationale, gated engine code |
| **G1** | per-session async transport | async tokio-`UdpSocket` engine (`StrOmTransport`) | **MET** | c002 port + c003 engine; loop turns over a real socket, tested |
| **G2** | trickle ICE | inbound `add_remote_candidate` + outbound `local_signals`; mapping tested | **MET (plumbing; connectivity gated)** | c004; srflx/STUN honestly browser-gated |
| **G3 (part)** | DTLS-connected milestone | crypto install + `wait_for_connected`; state path | **MET (milestone; two-peer proof gated)** | c005; peerless timeout proven, two-peer `#[ignore]` |
| **G4** | enable sovereign gate | **not flipped** — reached connected, not media | **CORRECTLY DEFERRED** | gate stays `hosted`; phase-21 flips it only when media flows |
| **G5** | live cross-node / OIDC | re-affirmed carried (no live infra) | **MET (re-affirmed deferred)** | c006; LiveKit `realtime`-gated, OIDC on ADR-004 |
| **close** | seed + sign-off | phase-21 seeded; §6 / CHANGELOG / sign-off; gates green | **MET** | c006 |

**Score: 6 / 6 delivered** against the operator-set scope. The two intentionally-partial
goals (G3 full = phase-21 RTP; G4 = flip only when media flows) were delivered as a *proven
milestone + a correct deferral*, not faked — the honest outcome, seeded into phase-21.

## Delivered Changes (6)

- **Decision:** c001 (ADR-005 — the `MediaTransport` port, gating all engine code).
- **Engine (built up incrementally):** c002 (port trait, impl-free), c003 (async per-session
  str0m loop), c004 (trickle ICE), c005 (DTLS crypto + connected milestone).
- **Close:** c006 (phase-21 seed + SECURITY §6 / CHANGELOG / sign-off + clean gate re-run).

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA gate run | 4 (c002–c005; c001 ADR + c006 close docs-only, skipped per contract) |
| First-pass gate pass rate | **4 / 4 (100%)** |
| Changes that BLOCKED then passed | 0 |
| Changes requiring code refinement after a gate | 0 |

### Recurring patterns

- **Zero QA BLOCKs this phase** — the spec-delta-before-gate + files-≤500 discipline (learned
  in 18/19) held. Issues were caught earlier, at the **compiler + standing clippy gates**,
  before the QA gate: an unreachable match arm (`IceConnectionState` fully covered), `Candidate`
  has no `FromStr` (→ `from_sdp_string`), a missing `tokio-stream` `sync` feature, `doc_markdown`
  on `local_addr` (recurring), and `struct_field_names` on `SessionMeta` (`#[allow]` w/ rationale).
- **Reading the vendored crate source paid off repeatedly** — str0m 0.21 + `is-0.10.0` +
  `str0m-proto-0.6.0` gave exact API (`Input`/`Output`/`net::Receive`, `Event::Connected`,
  the 5 `IceConnectionState` variants, `Candidate::from_sdp_string`, `from_feature_flags()`),
  avoiding guess-and-compile churn.
- **A near-miss on file size:** `session.rs` grew to exactly 498 lines by c005. The 500-line
  limit forced the earlier `ice.rs` extraction (c004) — without it c005 would have breached.
  Watch this at the start of phase-21 (RTP forwarding will grow the engine further → split first).

## Technical Debt Introduced / Carried

1. **RTP forwarding** — the actual media loop (`Event::MediaData` relay, per-room fan-out,
   PLI) is phase-21 (by design, ADR-005). `SFU_MODE=sovereign` stays gated off until it lands.
2. **Answerer-only port** — `MediaTransport` exposes only `create_session(offer)→answer`. The
   offerer/peer role is needed to un-gate the two-peer DTLS-connected proof (`#[ignore]`d) and
   for RTP — phase-21 G1.
3. **`session.rs` at 498/500 lines** — split it (e.g. extract the driver loop) before adding
   RTP in phase-21, or it breaches immediately.
4. **LiveKit live proof + admin-ui OIDC** — carried (no live infra this phase): LiveKit
   `realtime`-feature-gated; OIDC blocked on ADR-004 acceptance + an IdP.

## Lessons Captured

- **A hidden architecture question is worth an ADR, even mid-implementation.** The seed said
  "replace the signaler"; grounding it revealed the media engine had no port. ADR-005 (a new
  `MediaTransport` port, keeping signaling separate) was the load-bearing decision — building
  first would have overloaded `MediaSignaler`. Same pattern as ADR-004.
- **Translate the library's own canonical loop, don't reinvent it.** c003's async driver is a
  faithful async translation of str0m's `examples/http-post.rs` sans-I/O loop — reading the
  reference example is what made the `select!`-over-deadline structure correct on the first try.
- **When you can't prove it honestly, gate it and say why — precisely.** Reaching
  `Event::Connected` needs an offerer peer the port doesn't model; rather than a fake pass, the
  two-peer proof is `#[ignore]`d with the exact reason, and a *different* test proves the
  reachable part (crypto install + wait mechanic don't hang/panic). The gate is off; the doc
  says connected ≠ media.
- **Split before the file breaches, not after.** `session.rs` hit 498/500; the c004 `ice.rs`
  extraction is why c005 fit. Treat the limit as a leading indicator — extract at ~450.
- **Process-global init needs a `Once` + a matching `Default`.** The crypto provider installs
  once per process; guarding with `std::sync::Once` and routing `Default` through `new()` kept
  multiple `StrOmTransport`s (test + gateway) safe.

## Recommended Next Phase

**phase-21-sovereign-rtp-forwarding** (already seeded in c006) — turn the connected-milestone
engine into one that actually moves media:

1. **Offerer/peer role** — add an offer-creating path (port or peer harness) and un-`#[ignore]`
   the two-peer DTLS-connected proof (two sessions over loopback reach `Connected`).
2. **RTP forwarding** — handle `Event::MediaData` and forward via `rtc.writer(mid).write` to
   the room's other peers; add the RTP method ADR-005 deferred. **Split `session.rs` first**
   (it's at 498 lines).
3. **Per-room fan-out + PLI** — a room registry + keyframe handling so N peers exchange media.
4. **Enable the sovereign gate** — flip `SFU_MODE=sovereign` to a live path **only** once
   media flows end-to-end; cover its boundary in SECURITY §1–§5 first.
5. **Live proofs (carried)** — LiveKit `realtime` data source vs a live server; admin-ui OIDC
   once ADR-004 is Accepted + an IdP is stood up.

Do not advertise any of these as shipped until each functions end-to-end or is re-affirmed
deferred — the discipline that carried phases 16–20.
