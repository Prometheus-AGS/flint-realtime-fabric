# Goals — phase-17-plane-completion-and-release-audit

> Seeded from: phase-16-production-hardening reflection (2026-07-06)
> Source: `.kbd-orchestrator/phases/phase-16-production-hardening/reflection.md`
> → "Recommended Next Phase"

Phase-16 closed every CRITICAL/HIGH from the phase-15 audit (4 goals MET, G2
media/federation PARTIAL-by-design). It left three honest gaps: no independent
re-audit closing the "no CRITICAL on re-run" criterion, the artifact-refiner QA gate
was never wired in, and several planes were deliberately deferred rather than built.
This phase **verifies the release claim, wires quality enforcement, then completes the
deferred planes in dependency order — cheapest/pure-Rust first**.

Do not advertise any plane as shipped until its re-audit finding is closed.

---

## G1 — Verify the release claim (do first, gates the rest)

Phase-16's completion criterion ("a re-run of the production-readiness audit returns no
CRITICAL and a materially reduced HIGH count") was asserted per-goal but never
independently re-run.

- **G1.1** Re-run the production-readiness audit independently against real code (not
  against the phase-16 change list). Produce a fresh finding count.
- **G1.2** Confirm **zero CRITICAL** and a materially reduced HIGH count vs. the
  phase-15 baseline (7 CRITICAL / 15 HIGH). Any surviving CRITICAL blocks this phase.
- **G1.3** Confirm the CI gates actually run and pass on a clean checkout:
  `cargo fmt --check`, `clippy --workspace -- -D warnings -W clippy::pedantic
  -W clippy::unwrap_used`, `cargo check --workspace`, `cargo test --workspace`.

**Exit:** An independent re-audit report exists with no CRITICAL; CI is green on a
fresh clone.

---

## G2 — Wire the QA gate (process debt from phase-16)

Phase-16 ran 0/26 changes through artifact-refiner; quality rode on the Rust gates
alone, so the reflection's QA table was empty.

- **G2.1** Wire artifact-refiner into `/kbd-execute` (per-change QA gate before archive)
  so every phase-17 change is validated against `.kbd-orchestrator/constraints.md`.
- **G2.2** Optionally run artifact-refiner retroactively across the phase-16 changes
  before tagging a release, to backfill the skipped QA.

**Exit:** New changes pass the QA gate as part of execute; the gate is no longer skipped.

---

## G3 — Complete the deferred planes (cheapest / pure-Rust first)

From the phase-16 "Deferred to future phases" set. Order by external-dependency cost.

- **G3.1** `EntityService` gateway server (pure Rust, no SFU/federation dependency) —
  wire `GetEntity`/`WatchEntity` against the store; register in the gateway.
- **G3.2** `AuthzService` gateway server (pure Rust) — wire `Check`/`WriteRelation`/
  `DeleteRelation` against Keto; register in the gateway. Removes the "use the CLI
  directly" caveat from the API reference.
- **G3.3** str0m sovereign SFU: real WebRTC (Rtc/SDP/ICE negotiation, RTP forwarding),
  fixing the `from_session`/`to_session` route — then flip `SFU_MODE=sovereign` on.
- **G3.4** Federation: Matrix inbound + ATProto outbound protocol impls; LiveKit
  cross-node inbound relay. Only after the pure-Rust planes land.
- **G3.5** Finish Dart transport bindings via `uniffi-bindgen-dart` so the mobile SDK
  story (Swift/Kotlin/Dart) is complete and the package exposes a real API.

**Exit:** Every plane the gateway advertises either functions end to end or remains
honestly labeled; the pure-Rust services (Entity/Authz) are live; Dart SDK is usable.

---

## Phase completion criteria

- Independent production-readiness re-audit returns **no CRITICAL**; HIGH count
  materially reduced and each survivor triaged.
- artifact-refiner QA gate runs on every phase-17 change.
- `EntityService`/`AuthzService` reachable through the Rust + TS SDKs; the API
  reference's "proto-only" rows updated to "live".
- Remaining deferrals (str0m/federation) are either shipped or re-affirmed as deferred
  with an updated rationale — no drift back to "healthy but does nothing."

## Non-goals

- Net-new features beyond closing the phase-16 deferrals and the re-audit findings
  (YAGNI).
- Re-opening settled phase-16 decisions (ADR-001 CRDT, ADR-003 FFI toolchains) without
  a new finding forcing it.
