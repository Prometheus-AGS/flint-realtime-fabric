# Plan — phase-17-plane-completion-and-release-audit

> Backend: OpenSpec · Source: `assessment.md` (independent re-audit) + 3 operator scope
> decisions (2026-07-06).
> Theme: **verify + pure-Rust completion.** The security boundary already holds (zero
> CRITICAL/HIGH survive); this phase closes the one MEDIUM residual, wires the QA gate
> phase-16 skipped, and completes the planes that need **no external SFU/federation
> dependency**.

## Scope decisions (operator, 2026-07-06)

1. **Big planes deferred.** str0m real WebRTC (A10) and federation impls (A11) are **out
   of scope** for phase-17 — they get a focused later phase. This phase refreshes their
   deferral rationale and fixes their doc drift, nothing more.
2. **CLI: build the missing commands.** Add real CDC slot management + broker-offset
   inspect so the CLI matches its advertised surface (A7 = build, not re-scope).
3. **Dart in scope.** `uniffi-bindgen-dart` is available in CI — generate the bindings,
   commit them, fix the `GENERATED.md` drift (A8 = build).

## Ordered change list (10 changes)

Ordering rationale: **close G1 residual first** (security hygiene), **then wire the QA
gate** (so every subsequent change is validated), **then pure-Rust servers** (highest
value, lowest cost — they flip the API-reference "proto-only" rows to "live"), **then
SDK/CLI/FFI coverage**, **then Dart**, **then the honest re-affirmation of deferrals**.

| # | Change | Goal | Gap | Agent | Size |
|---|--------|------|-----|-------|------|
| **p17-c001** | Fix `compose.override.yml` footgun: gitignore it (or ship `.example` + top-of-file deploy-base guard) and remove the committed dev-secret literal | G1 | A1 (MEDIUM) | security-reviewer | S |
| **p17-c002** | Make `JWT_ISSUER` mandatory-in-prod: boot-time validation error when unset outside dev; document in ENVIRONMENT.md | G1 | A2 (LOW) | rust-reviewer | S |
| **p17-c003** | Author `.kbd-orchestrator/constraints.md` + wire `/refine-validate` per-change into the execute loop so phase-17 changes are QA-gated | G2 | A3 | general-purpose | M |
| **p17-c004** | `EntityService` gateway server (pure Rust): tonic impl of GetEntity/WatchEntity backed by a store; register in `spawn_grpc_server` | G3.1 | A4 | rust-reviewer | M |
| **p17-c005** | `AuthzService` gateway server (pure Rust): tonic impl of Check/WriteRelation/DeleteRelation delegating to the existing `KetoAuthzProvider`; register in gateway | G3.2 | A5 | rust-reviewer | M |
| **p17-c006** | Bind Sync/Agent/Signal (+ new Entity/Authz) clients in **frf-sdk-rust**; add resilient-subscribe + `ack` to **frf-ffi** so mobile gets reconnect/replay parity | G3 adj | A6 | rust-reviewer | M |
| **p17-c007** | `frf-cli`: add CDC slot management (create/drop) + broker-offset **inspect** command, matching the advertised surface | G3 adj | A7 | rust-reviewer | S–M |
| **p17-c008** | Finish **Dart** bindings via `uniffi-bindgen-dart` (run build, commit generated `lib/src/rust/*.dart`); fix stale `GENERATED.md` (flutter_rust_bridge → uniffi-bindgen-dart) | G3.5 | A8 | dart-build-resolver | M |
| **p17-c009** | Update **API-REFERENCE.md** (Entity/Authz rows proto-only → live) + fix str0m `sfu.rs:30-33` doc drift (references non-existent `process_offer`/`process_ice`); refresh CHANGELOG | G3.3 | A9 | doc-updater | XS–S |
| **p17-c010** | **Re-audit + release sign-off:** re-run the deterministic gates + a security spot-check on a clean checkout after c001–c009; re-affirm str0m/federation deferrals with updated rationale; update README phase status | G1/G3 | — | code-reviewer | S |

**Deferred (not in this phase, documented):** A10 str0m WebRTC, A11 Matrix inbound /
ATProto outbound / LiveKit cross-node inbound, A12 admin-ui real OIDC login flow. These
carry to a future **phase-18 media-federation-and-auth-flow** phase. c009/c010 keep their
labels honest so nothing reads as "shipped."

## Exit criteria (from goals.md, refined)

- **G1:** MEDIUM (compose.override footgun) closed; `JWT_ISSUER` enforced in prod; final
  gates green on a clean checkout (c010).
- **G2:** `constraints.md` exists and `/refine-validate` runs on every phase-17 change.
- **G3:** `EntityService`/`AuthzService` reachable through the Rust + TS SDKs; API
  reference "proto-only" rows updated to "live"; CLI matches its advertised surface;
  Dart SDK exposes a real generated API.
- Remaining deferrals (str0m/federation/admin-ui login) re-affirmed with rationale — no
  "healthy but does nothing," no doc drift.

## Notes for execute

- c003 is a **prerequisite for QA-gating the rest** — apply it early; once
  `constraints.md` exists, c004+ each run through `/refine-validate` before archive.
- c004/c005 are the highest-leverage changes: pure Rust, backing logic already exists
  (`KetoAuthzProvider` for c005; a store for c004), and they retire the two biggest
  "proto-only" caveats in the public API surface.
- c010 is the phase's own mini-re-audit — it closes the loop the assessment opened.
