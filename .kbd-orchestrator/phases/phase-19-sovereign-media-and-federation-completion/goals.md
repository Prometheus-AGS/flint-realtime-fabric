# Goals — phase-19-sovereign-media-and-federation-completion

> Seeded from: phase-18-media-federation-and-auth-flow reflection (2026-07-07)
> Source: `.kbd-orchestrator/phases/phase-18-media-federation-and-auth-flow/reflection.md`
> → "Recommended Next Phase"

Phases 16–18 shipped a release-ready **hosted** deployment (security hardened + audited,
all six proto services live, SDK/CLI/FFI parity) and completed the *cheap and
self-contained* deferred planes (str0m signaling fixed, Matrix inbound, ATProto outbound
capability, admin-ui token hardening, Dart shim). What remains are the **XL and
external-dependency items** phase-18 explicitly deferred here.

This phase completes the sovereign media plane and the last federation directions, plus
the small wiring/cleanup follow-ups. **The largest item — the str0m full SFU media loop —
may warrant its own sub-phase or split; decide during assess/plan.**

**Discipline (carried from 16–18):** do not advertise any plane as shipped until it
functions end-to-end or is re-affirmed deferred. No "healthy but does nothing." Update
`docs/SECURITY.md` §6 and the CHANGELOG as each lands, and re-run the full gate suite at
phase close.

---

## G1 — str0m full sovereign SFU (largest / highest-risk; likely its own focus)

Phase-18 (c006) proved the negotiation round-trip on str0m 0.21 and documented the
remaining media-loop work in `crates/frf-media-str0m/SPIKE-FINDINGS.md`. This goal builds
the live media plane on that foundation.

- **G1.1** Per-session `Rtc` state machine with a real UDP socket and the sans-I/O event
  loop: `handle_input(Input::Receive/Timeout)` + `poll_output()` → `Transmit/Timeout/Event`.
- **G1.2** Trickle ICE over the signaling channel: pipe inbound `IceCandidate` envelopes
  into `add_remote_candidate`, and emit the SFU's local candidates back through
  `SignalService`. DTLS/SRTP is handled by str0m.
- **G1.3** RTP forwarding between session peers (`Event::MediaData` → `writer(mid).write`),
  keyed by mid/track, with per-room fan-out topology and keyframe (PLI) handling.
- **G1.4** Replace the channel-only `StrOmSignaler` architecture with per-session async
  transport tasks; consider whether the `MediaSignaler` port needs a media surface.
- **G1.5** Flip `SFU_MODE=sovereign` on **only** when media flows end-to-end (browser
  reports `connected` + media relays); otherwise keep it gated and re-affirm.

**Exit:** two peers exchange real media through the sovereign SFU, proven by a test, OR
the plane is re-affirmed deferred with an updated rationale. Biggest risk (per the spike):
the sans-I/O UDP/ICE/DTLS transport loop — hard to test without a real browser.

---

## G2 — LiveKit cross-node inbound relay

Today LiveKit inbound is in-process only (no subscription to the LiveKit server data
channel), so signals from remote nodes never surface locally.

- **G2.1** Add a LiveKit realtime data-channel client that listens for server-originated
  data events and feeds them into `subscribe_signals` for true cross-node inbound relay.

**Exit:** a signal published on one node reaches a subscriber on another node, OR the item
is re-affirmed deferred.

---

## G3 — Wire ATProto outbound into the gateway (small)

Phase-18 (c008) implemented + tested the ATProto PDS write *capability*, but `main.rs`
builds the bridge inbound-only.

- **G3.1** Add PDS identity config (service URL, identifier, app-password via env/secret
  manager) and call `AtProtoBridge::with_writer` in `main.rs` when configured.
- **G3.2** Document the new env vars in `ENVIRONMENT.md`; never commit the app-password.

**Exit:** with PDS config set, the gateway's ATProto bridge writes federated events to the
PDS end to end.

---

## G4 — Full admin-ui OIDC login (blocked on an IdP)

Phase-18 (c005) hardened the token gate; a real interactive login needs an IdP.

- **G4.1 (decision/infra)** Stand up an IdP — Ory Kratos/Hydra in compose — OR add a
  flint-gate authorization-code endpoint. **This is the load-bearing prerequisite;
  nothing in the frontend flow can proceed until it exists.**
- **G4.2** admin-ui OIDC authorization-code + PKCE flow: redirect → `/callback` handler →
  token exchange → session; add refresh + logout. May require a router change (the app
  uses a hand-rolled hash router today).

**Exit:** an operator logs in through a real flow (not a pasted token), OR G4 is
re-affirmed deferred pending the IdP decision.

---

## G5 — Follow-ups & cleanup (small)

- **G5.1** Dart async transport: revisit when `uniffi-bindgen-dart` fixes async/callback
  codegen (0.1.3 is the latest that does not); regenerate and **remove the c009
  generated-file patch** when the generator is fixed, or land the full hand-lowering.
- **G5.2** Admin-ui lint debt: fix the 2 pre-existing lint errors
  (`e2e/p7-smoke.spec.ts`, `entities/useEntitySubscription.ts`).

**Exit:** the Dart async surface works or is re-affirmed deferred with the upstream
tracking reference; admin-ui `pnpm lint` passes clean.

---

## Phase completion criteria

- Each goal either **functions end-to-end** (proven, not asserted) or is **re-affirmed
  deferred** with fresh rationale in `docs/SECURITY.md` §6 + CHANGELOG.
- A re-run of the release gates (fmt / clippy pedantic + unwrap_used / check / test) stays
  green; each code change passes the QA gate.
- No "healthy but does nothing": any newly-enabled `SFU_MODE=sovereign` / cross-node /
  ATProto-outbound / OIDC path actually moves media/events/auth, or stays gated off.

## Non-goals

- Net-new features beyond the deferred items (YAGNI).
- Re-opening settled decisions (ADR-001 CRDT, ADR-003 FFI toolchains) without a new finding.
- Re-auditing the already-signed-off hosted release surface (phases 16–18).

## Suggested ordering for `/kbd-plan`

G3 (small, self-contained wiring) + G5.2 (lint) first; then G4.1 (the IdP decision — it
gates G4.2); then G2 (LiveKit inbound); then **G1 (str0m full SFU) as the phase's main
thrust or its own sub-phase**, since it is the largest and riskiest. G5.1 (Dart) is
toolchain-gated — do it if a fixed generator is available, else re-affirm deferred.
