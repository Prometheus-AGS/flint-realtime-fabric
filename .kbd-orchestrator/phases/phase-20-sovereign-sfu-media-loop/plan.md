# Plan — phase-20-sovereign-sfu-media-loop

> Stage: Plan · 2026-07-08 · Backend: **OpenSpec** (`change_backend: openspec`)
> Source: `assessment.md`. Operator decisions locked (below).
> Ordering: **ADR for the load-bearing unknown first, then capability built + unit-tested,
> browser/infra proofs honestly integration-gated** — the phase-16–19 discipline.

## Operator decisions (locked this stage)

1. **SFU architecture:** a **new `MediaTransport` port** — keep `MediaSignaler` for
   signaling; the per-session `Rtc` + UDP + RTP engine implements a separate port composed
   alongside it (honors one-port-per-adapter). ADR first, before any engine code.
2. **G3 scope:** phase-20 stops at a **proven milestone** — per-session async loop +
   trickle-ICE wiring + reaching **DTLS-connected** — and **splits full RTP forwarding /
   per-room fan-out / PLI into a dedicated phase-21**.
3. **G5 infra:** **no live infra** this phase. G5.1 (LiveKit live proof) stays
   integration-gated behind the `realtime` feature; G5.2 (admin-ui OIDC) stays blocked on
   ADR-004 acceptance + an IdP. Both carried forward honestly (a re-affirmation change).

## Ordered changes

Each is one OpenSpec change (`/opsx:new` → proposal + spec delta + tasks), walked one task
per turn via `/kbd-apply` from the repo root. Spec delta written **before** the QA gate
(P1 `openspec validate`, MUST-on-line-after-header). QA gate per change unless docs-only.

| # | Change ID | Goal | Type | QA | Summary |
|---|-----------|------|------|:--:|---------|
| c001 | `p20-c001-media-transport-port-adr` | G1 arch | docs | skip | ADR-005: new `MediaTransport` port; recommend + rationale; gates all engine code. |
| c002 | `p20-c002-media-transport-port-trait` | G1 | feat | yes | Define the `MediaTransport` port trait in `frf-ports` (no impl); newtype/session types. |
| c003 | `p20-c003-str0m-async-session-loop` | G1 | feat | yes | Async per-session `Rtc`+tokio-`UdpSocket` task (evolve the c006 `TransportLoop`); add `net`/`time` features; async loop test. |
| c004 | `p20-c004-trickle-ice-wiring` | G2 | feat | yes | Pipe `IceCandidate` envelopes ↔ `add_remote_candidate`; emit local candidates; unit-test the plumbing; connectivity integration-gated. |
| c005 | `p20-c005-dtls-connected-milestone` | G3(part) | feat | yes | Drive DTLS to `Connected` on the async session; surface the connection event; **RTP forwarding deferred to phase-21**. |
| c006 | `p20-c006-g5-reaffirm-and-phase-21-seed` | G4/G5/close | docs | skip | Re-affirm G5.1/G5.2 gated; keep `SFU_MODE=sovereign` off (media not yet forwarded); seed phase-21-sovereign-rtp-forwarding; SECURITY §6 / CHANGELOG / sign-off + gate re-run. |

**6 changes.** c001 is the gating ADR; c002–c005 build the async media session up to
DTLS-connected; c006 closes honestly and seeds phase-21 for the RTP forwarding.

---

## Change detail

### c001 — MediaTransport port ADR (G1 arch) · docs · QA:skip
- **Files:** `docs/decisions/adr-005-media-transport-port.md` (new).
- **Tasks:** state the constraint (`MediaSignaler` is signaling-only; `StrOmSignaler` has no
  `Rtc`/UDP/RTP); present the 3 options (new port / extend / internal engine); **recommend
  a new `MediaTransport` port**; define its rough surface (session create/answer, ICE
  candidate in/out, connection-state events; **RTP forwarding is phase-21**); note the
  composition point in the gateway and that `SFU_MODE=sovereign` stays gated off until media
  flows.
- **Exit:** the port architecture is decided and reviewable; c002 implements it.

### c002 — MediaTransport port trait (G1) · feat · QA:yes
- **Files:** `crates/frf-ports/src/media_transport.rs` (new; keep ≤500 L), `frf-ports/src/lib.rs`.
- **Tasks:** define the `MediaTransport` trait per ADR-005 (no implementation) — e.g.
  `create_session(offer) -> answer`, `add_remote_candidate`, `poll_local_candidate`,
  `connection_state`; associated error via `PortError`; `#[non_exhaustive]` public enums;
  newtype session IDs. A trait-level doc test / compile check.
- **Exit:** the port exists in `frf-ports` with no adapter dependency; clean gate.
- **Constraint:** `frf-ports` stays implementation-free (dependency rule).

### c003 — str0m async per-session loop (G1) · feat · QA:yes
- **Files:** `crates/frf-media-str0m/src/session.rs` (new; the async engine),
  `Cargo.toml` (add tokio `net`,`time`,`macros`), `lib.rs`. Keep the c006
  `transport_spike.rs` as the proven reference.
- **Tasks:** evolve the blocking `TransportLoop` into an **async per-session task**: tokio
  `UdpSocket`, `select!` over `recv_from` and the `poll_output` `Timeout` deadline,
  `Transmit` → `send_to`, events surfaced on a channel. Implement (the transport slice of)
  `MediaTransport`. Async test that the loop turns over a real tokio socket (extends the
  c006 proof to the async/multi-poll shape). No library `unwrap`/`expect`.
- **Exit:** a per-session async task drives a real `Rtc` over async UDP, proven by a test.

### c004 — trickle ICE wiring (G2) · feat · QA:yes
- **Files:** `crates/frf-media-str0m/src/session.rs` (or an `ice.rs` sibling if it nears 500 L).
- **Tasks:** map inbound `SignalKind::IceCandidate` envelopes → `rtc.add_remote_candidate`;
  emit the session's local candidates back out as `IceCandidate` envelopes through the
  signaling path. Unit-test the envelope↔candidate mapping (parse/format) deterministically.
  Real connectivity (host/srflx/STUN gathering + checks) is **integration/browser-gated** —
  documented, `#[ignore]`/feature-flagged.
- **Exit:** ICE candidate plumbing wired + unit-tested; connectivity proof integration-gated.

### c005 — DTLS-connected milestone (G3 part) · feat · QA:yes
- **Files:** `crates/frf-media-str0m/src/session.rs`.
- **Tasks:** drive the session far enough that str0m completes the **DTLS handshake to
  `Connected`** and surface that connection-state event; assert the state machine reaches
  connected in a test where feasible, else document the browser-gated exit.
  **RTP `MediaData` forwarding, per-room fan-out, and PLI are explicitly NOT in scope —
  phase-21.** `SFU_MODE=sovereign` stays gated off (connected ≠ media flowing).
- **Exit:** the async session reaches DTLS-connected (or the exact browser-gated blocker is
  documented); no media forwarded yet.

### c006 — G5 re-affirm + phase-21 seed + close · docs · QA:skip
- **Files:** `.kbd-orchestrator/phases/phase-21-sovereign-rtp-forwarding/` (seed),
  `docs/SECURITY.md` §6, `CHANGELOG.md`, `docs/PHASE-20-SIGNOFF.md` (new).
- **Tasks:** (1) seed phase-21 (full RTP forwarding + per-room fan-out + PLI, building on
  the c003–c005 connected session; plus the carried G5.1 LiveKit live proof and G5.2 OIDC).
  (2) Re-affirm G5.1/G5.2 gated (no live infra this phase). (3) SECURITY §6: str0m now
  reaches DTLS-connected, **media still deferred (phase-21)**, `SFU_MODE=sovereign` still
  gated off. (4) CHANGELOG + sign-off; re-run the release gate suite.
- **Exit:** phase-20 signed off; phase-21 seeded; no plane advertised beyond what it does.

---

## Sequencing & rationale

- **c001 (ADR) first** — the port-architecture decision gates every engine change (the G1
  blocker from the assessment); exactly as ADR-004 gated OIDC in phase-19.
- **c002 → c003 → c004 → c005** — build the async media session incrementally: port trait,
  async loop, ICE wiring, DTLS-connected. Each is testable/gate-passable on its own; a spill
  on the browser-gated pieces (c004/c005) doesn't strand the phase.
- **c005 stops at DTLS-connected** — the honest phase-20 milestone; full RTP is c006's
  phase-21 seed.
- **c006 last** — closes and seeds phase-21; keeps the gate off (connected ≠ media).

## Phase exit criteria

- The `MediaTransport` port exists (c002), an async per-session str0m loop drives a real
  socket (c003), ICE candidates are wired (c004), and the session reaches DTLS-connected
  (c005) — each proven or honestly integration-gated.
- Release gate suite green; each code change passes the QA gate.
- **No "healthy but does nothing":** `SFU_MODE=sovereign` stays gated off — the phase reaches
  *connected*, not *media forwarded*; RTP is honestly deferred to phase-21. G5 carried,
  gated.

## First change to apply

`p20-c001-media-transport-port-adr` — the ADR that unblocks all G1–G3 engine work.
