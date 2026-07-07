# Reflection — phase-17-plane-completion-and-release-audit

> Generated: 2026-07-07
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **10 / 10 DONE**, all archived. Seeded from the phase-16 reflection's
> "Recommended Next Phase".

## Summary

This phase did exactly what phase-16's reflection asked: **verify the release claim
independently, wire the QA gate phase-16 skipped, then complete the deferred planes
cheapest-first.** All three of phase-16's honest gaps are now closed. The security
boundary was re-audited against real code (not the change list) and holds — zero CRITICAL,
zero HIGH survive in the production build. Every code change c004–c008 passed through the
newly-wired QA gate before archive. All six proto services are now live.

The phase's defining quality is that **the QA gate earned its place immediately**: its
first smoke-run (during c003) caught a real over-matching bug in its own secret check, and
that same gate then validated five substantive changes. Quality was measured, not asserted
— unlike phase-16, whose QA table was empty.

## Goal Achievement

| Goal | Title | Verdict | Evidence |
|------|-------|---------|----------|
| **G1** | Verify the release claim | **MET** | Independent re-audit ran as the assessment (3 parallel code-grounded audits + gates): zero CRITICAL / zero HIGH in the prod build. c010 re-ran fmt / check / clippy(pedantic+unwrap_used, incl. tests & dev-endpoints) / lib+integration tests — all green. `docs/RELEASE-SIGNOFF.md` records the result. |
| **G2** | Wire the QA gate | **MET** | c003 authored `.kbd-orchestrator/constraints.md` + `bin/qa-gate.sh` and wired it into `execution.md`. c004–c008 each ran it before archive; logs persisted under `.refiner/artifacts/`. Closes phase-16's 0/26 process debt. |
| **G3** | Complete the deferred planes (cheapest-first) | **MET (pure-Rust scope)** | `EntityService` (c004) + `AuthzService` (c005) gateway servers live; SDK/FFI/CLI parity (c006/c007); Dart CRDT bindings generated + committed (c008). The big planes (str0m WebRTC, federation, admin-ui login) were **operator-scoped out** and remain honestly deferred. |

**Score: 3 / 3 MET** against the scope the operator chose (verify + pure-Rust
completion; str0m WebRTC + federation explicitly deferred to a future phase). No goal
NOT-MET; no goal PARTIAL within the agreed scope.

## Delivered Changes (10)

- **G1 residual (security hygiene):** c001 (untrack `compose.override.yml`, remove
  committed dev-secret, ship `.example`), c002 (JWT_ISSUER mandatory in prod build + test).
- **G2 (QA gate):** c003 (constraints.md + qa-gate.sh + FAIL-path contract + smoke test
  that caught the S1 bug).
- **G3 (pure-Rust planes):** c004 (`EntityService` server — new `EntityStore` port,
  auth-guarded use-case, in-memory store, 9 tests), c005 (`AuthzService` server —
  tenant-scoped, no tuple logging, 6 tests), c006 (Rust SDK binds all 5 services; FFI
  resilient subscribe + ack; TS/Go/C# Entity+Authz), c007 (`frf broker offsets`, `frf cdc
  slot create|drop`), c008 (Dart bindings via `uniffi-bindgen-dart`).
- **Docs & sign-off:** c009 (API-reference Entity/Authz → live, str0m doc-drift fix,
  CHANGELOG), c010 (clean-checkout gate re-run, security spot-check, SECURITY.md §6
  deferred-planes posture, README status, RELEASE-SIGNOFF.md).

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA gate run | 5 (c004–c008; per the contract, c001/c002 pre-gate, c003 is the gate, c009 docs-only, c010 verification) |
| First-pass gate pass rate | **5 / 5 (100%)** |
| Changes requiring refinement after a red gate | 0 |
| Gate self-test artifacts | 1 (the `p17-c002` log is c003's negative smoke-test — it deliberately planted a secret to prove the gate BLOCKS; not a real change failure) |

### Recurring constraint violations

None across changes. Every gate-run change was green on the first pass. Issues that *were*
caught happened at the standing Rust-gate layer **before** the QA gate (see Lessons) —
e.g. a `u32→i32` cast-wrap (c004), `private_interfaces` leak (c006), `needless_pass_by_value`
(c005/c006), `items after a test module` (c007). These are healthy: the gates caught them,
they were fixed, and the QA gate then passed clean.

## Technical Debt Introduced / Carried

1. **Dart async-transport bindings blocked upstream.** `uniffi-bindgen-dart` 0.1.3 emits
   type-incorrect Dart for async constructors and foreign callbacks, so
   `FrfFfiClient.connect`/`subscribe` don't type-check. The CRDT surface works; the gap is
   documented in `sdks/dart/GENERATED.md` and the generated file is analyzer-excluded (not
   hand-patched). Follow-up: re-run when the generator is fixed, or write a thin Dart shim.
2. **In-memory default stores.** `EntityService` uses an `InMemoryEntityStore` (the
   `SyncService` pattern). A persistent `frf-store-surreal` `EntityStore` impl is future
   work; the port exists so it's a drop-in.
3. **Big planes still deferred** (by operator choice): str0m sovereign WebRTC, Matrix/
   ATProto federation, LiveKit cross-node relay, admin-ui OIDC login. All gated off /
   labeled unimplemented; SECURITY.md §6 states their (absent) security posture.
4. **Full `cargo test --workspace` exceeds the 2-min local link budget** — runs in CI.
   Lib + key integration suites are verified locally.

## Lessons Captured

- **A QA gate must be adversarially tested before it's trusted.** c003's smoke-run found
  the gate's own S1 check over-matching a legitimate JWT test fixture. Verifying it in
  *both* directions (passes clean code, blocks a planted secret) was worth more than
  shipping an unvalidated gate — the exact mistake phase-16 made by never running one.
- **Layered gates catch different things.** The standing Rust gates (clippy pedantic,
  unwrap_used) caught every real correctness issue *before* the QA gate saw the change;
  the QA gate's value is enforcing the cross-cutting constraints (architecture, secrets,
  spec-delta) that clippy can't see. Both layers matter.
- **"Available in CI" ≠ "works."** The operator confirmed `uniffi-bindgen-dart` was
  available; it installed and generated — but `dart analyze` surfaced generator bugs.
  Running the real toolchain (not trusting the claim) is what surfaced the honest limit.
- **The pipeline-enforce hook reads `progress.json`, not intent** (again). Bumping
  progress and writing a `reflect`-referencing waypoint string in one command tripped the
  gate at 9/10 — same as phase-16. Bump progress first; touch the waypoint separately.
- **Grep for a forbidden token matches comments too.** The c010 security spot-check first
  "found" `DEV_NO_AUTH` in `compose.yml` — but only in comments stating its absence.
  Distinguish active settings (`^\s+KEY:`) from documentation before calling a regression.

## Recommended Next Phase

**phase-18-media-federation-and-auth-flow** — the explicitly-deferred big planes, now that
verify + pure-Rust completion is done. Ordered by dependency/risk:

1. **admin-ui interactive OIDC/flint-gate login** (pure frontend + existing JWT verify;
   no new external dependency) — removes the paste-a-JWT gate.
2. **str0m sovereign SFU real WebRTC** (`Rtc`/SDP/ICE/RTP) — the single largest item;
   consider a spike first.
3. **Federation**: Matrix inbound sync loop, ATProto outbound PDS writes, LiveKit
   cross-node inbound relay.
4. **Dart async-transport bindings** — revisit when `uniffi-bindgen-dart` fixes the
   async/callback codegen, or land the hand-written shim.

Do not advertise any of these as shipped until each functions end-to-end or is re-affirmed
deferred — the same discipline that carried phases 16–17.
