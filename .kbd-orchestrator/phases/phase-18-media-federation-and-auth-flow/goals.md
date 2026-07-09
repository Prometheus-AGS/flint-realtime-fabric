# Goals — phase-18-media-federation-and-auth-flow

> Seeded from: phase-17-plane-completion-and-release-audit reflection (2026-07-07)
> Source: `.kbd-orchestrator/phases/phase-17-plane-completion-and-release-audit/reflection.md`
> → "Recommended Next Phase"

Phases 16–17 shipped a release-ready **hosted** deployment: security boundary hardened
and re-audited (zero CRITICAL/HIGH), all six proto services live, SDK/CLI/FFI parity, QA
gate wired. What remains are the **explicitly-deferred big planes** — the media,
federation, and interactive-auth surfaces that phases 16–17 gated off rather than shipped
half-secured.

This phase completes them **in dependency/risk order — cheapest and lowest-external-risk
first**. Each plane is currently gated off or labeled unimplemented; each goal moves one
from "deferred" to "functions end-to-end" (or re-affirms deferral with fresh rationale).

**Discipline (carried from 16–17):** do not advertise any plane as shipped until it
functions end-to-end or is re-affirmed deferred. Every plane the gateway boots must either
work or be honestly labeled — no "healthy but does nothing." Extend `docs/SECURITY.md` §6
(deferred-planes posture) as each lands.

---

## G1 — admin-ui interactive login (do first: cheapest, no new external dependency)

The admin UI currently has a paste-a-JWT gate (`LoginGate`), not a real auth flow. The
gateway already verifies tokens via flint-gate JWKS — this is pure frontend + existing
verify, no new backend dependency.

- **G1.1** Implement an interactive OIDC / flint-gate login flow in `admin-ui`
  (redirect → token → session), replacing the paste-a-JWT `LoginGate`.
- **G1.2** Handle token lifecycle: store securely, refresh, and clear on logout.
- **G1.3** Keep the gateway's JWT verification unchanged — the UI mints/refreshes, the
  gateway still verifies. No unverified claims trusted downstream.

**Exit:** an operator logs in through a real flow (not a pasted token); the admin UI
carries a valid JWT it obtained itself; `docs/SECURITY.md` admin-ui note updated.

---

## G2 — str0m sovereign SFU: real WebRTC (largest item — spike first)

`frf-media-str0m` is signaling-only today (no `Rtc`/SDP/ICE/RTP); `SFU_MODE=sovereign` is
gated off with a warning. This is the single largest, highest-risk item.

- **G2.1 (spike)** Prove a minimal `str0m::Rtc` round-trip (SDP offer/answer + ICE +
  DTLS) between two sessions before committing to the full build.
- **G2.2** Implement per-session `Rtc` state machines driven by the gRPC `SignalService`
  (SDP/ICE in), and RTP forwarding between session peers (media out).
- **G2.3** Flip `SFU_MODE=sovereign` on only when media actually flows end-to-end;
  otherwise keep it gated and re-affirm the deferral.

**Exit:** two peers exchange real media through the sovereign SFU, OR the plane is
re-affirmed deferred with an updated, honest rationale.

---

## G3 — Federation (after the pure planes; external protocol dependencies)

- **G3.1** Matrix inbound: replace `stream::empty()` with a real sync loop (long-poll
  `/sync` or a Tuwunel client when available). Outbound already works.
- **G3.2** ATProto outbound: implement authenticated PDS writes
  (`com.atproto.repo.createRecord`). Inbound Jetstream already works.
- **G3.3** LiveKit cross-node inbound relay: subscribe to the LiveKit server data channel
  so signals from remote nodes surface locally (today inbound is in-process only).

**Exit:** each federation direction functions end-to-end, or is re-affirmed deferred with
rationale. Federated events land where a subscriber's JWT tenant matches (no per-boot
random tenant).

---

## G4 — Dart async-transport bindings (toolchain-gated)

Phase-17 shipped the Dart CRDT bindings; the async transport surface
(`FrfFfiClient.connect` / `subscribe`) is blocked by a `uniffi-bindgen-dart` 0.1.3
codegen bug.

- **G4.1** Check whether a newer `uniffi-bindgen-dart` fixes async-constructor /
  foreign-callback codegen; if so, regenerate and commit.
- **G4.2** If still broken upstream, land a thin hand-written Dart shim over the sync FFI
  for connect/subscribe/ack, so the mobile transport surface is usable.

**Exit:** Dart clients can connect/subscribe/ack (via fixed codegen or a shim), or the
gap is re-affirmed deferred with the upstream tracking reference.

---

## Phase completion criteria

- Each plane above either **functions end-to-end** (proven, not asserted) or is
  **re-affirmed deferred** with fresh rationale in `docs/SECURITY.md` §6 + CHANGELOG.
- A re-run of the release gates (fmt / clippy pedantic + unwrap_used / check / test) stays
  green; each code change passes the QA gate wired in phase-17.
- No "healthy but does nothing": any newly-enabled `SFU_MODE`/`FEDERATION_ENABLED` path
  actually moves media/events, or stays gated off.

## Non-goals

- Net-new features beyond the four deferred planes (YAGNI).
- Re-opening settled decisions (ADR-001 CRDT, ADR-003 FFI toolchains) without a new
  finding forcing it.
- Re-auditing the already-signed-off hosted release surface (phases 16–17).

## Suggested ordering for `/kbd-plan`

G1 (cheapest, unblocks operator UX) → G2 spike (G2.1) before committing to the full SFU
build → G2 full → G3 (external deps) → G4 (toolchain-gated). Consider splitting G2 (str0m)
into its own sub-phase if the spike reveals it's larger than the rest combined.
