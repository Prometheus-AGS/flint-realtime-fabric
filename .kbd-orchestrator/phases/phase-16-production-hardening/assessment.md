# Assessment — phase-16-production-hardening

> Authored: 2026-07-06
> Assessor: kbd-assess
> Baseline: fresh phase; goals = the release-blocking findings from the phase-15
> production-readiness audit. This assessment establishes the concrete starting
> state and confirms nothing has been fixed yet.

---

## Summary

phase-16 was seeded directly from the phase-15 production-readiness audit
(`../phase-15-stage10-live-run-and-retries/assessment-production-readiness.md`:
7 CRITICAL, 15 HIGH, 24 MEDIUM, 15 LOW). This assess pass verifies each G1 target
against live code and spot-checks G2–G5. **All targeted findings are still open** —
expected, as no execution has run. The workspace compiles clean (`cargo check
--workspace` = exit 0), giving a stable starting point.

One new structural fact surfaced that the plan must account for: the auth bypass is
spread across **three** compose files, not one.

---

## Cross-Tool Progress

NONE — no cross-tool activity recorded since phase creation. `progress.json` shows
`changes_total: 0, changes_completed: 0`, `assessment_complete: false`.

---

## Implementation Status vs. Goals

### G1 — Security boundary (all OPEN, do first)

- **G1.1 — compose auth bypass**: **OPEN.** Confirmed present in THREE files:
  `compose.yml:9` (`CARGO_FEATURES: dev-endpoints`) + `:29` (`DEV_NO_AUTH: "true"`);
  `compose.override.yml:18` (`DEV_NO_AUTH: "true"` — auto-merged on plain
  `docker compose up`); `compose.ci.yml:20,35` (both). `compose.yml` is
  production-shaped (real Keto, flint-gate JWKS, JWT_AUDIENCE, live CDC) yet compiles
  in + activates the bypass. **Plan nuance:** CI legitimately needs the bypass — the
  fix must keep `compose.ci.yml`/override functional for dev/CI while making the
  production-shaped `compose.yml` secure by default.
- **G1.2 — tenant-equality assertion**: **OPEN.** `frf-app/src/subscribe.rs:52,78` and
  `publish.rs:50` read `claims.tenant_id` but never compare it to the target
  channel/envelope `tenant_id`. No mismatch rejection exists.
- **G1.3 — enforce `unwrap_used`**: **OPEN.** `unwrap_used` is not referenced in any
  `Cargo.toml`, workspace manifest, `ci.yml`, or `dagger/codegen.ts`. Gate is
  documented-only. (~50 real library unwraps remain, concentrated in frf-crdt/redb.)
- **G1.4 — JWT `iss` verification**: **OPEN.** No issuer/`iss` validation in
  `frf-identity-ory/src/`.
- **G1.5 — rate-limit / CORS / body limit**: **OPEN.** No `.layer(...)`, `CorsLayer`,
  `RateLimit`, or `RequestBodyLimit` on the gateway router (`lib.rs`, `main.rs`).
- **G1.6 — externalize signing secret**: **OPEN** (secret still in compose; verify
  during execution).
- **G1.7 — Cedar allow-all/deny-all**: **OPEN** (per phase-15 finding #H3/#11).

### G2 — Make advertised planes real (OPEN)

- **G2.1 str0m SFU** (C4): OPEN — no WebRTC; wired unconditionally.
- **G2.2 admin-UI ↔ gateway transport** (C2/C3/H9/H10/H11): **OPEN.** No `rust-embed`/
  `ServeDir`, no `tonic_web`/`accept_http1`, no `/ws/v1/signal` route in the gateway.
- **G2.3 LiveKit cross-node** (H4), **G2.4 federation tenant IDs** (H5),
  **G2.5 Matrix inbound / ATProto outbound** (#10/#27): OPEN per phase-15.

### G3 — Missing deliverables (OPEN)

- **G3.1 `frf-sdk-rust`**: **STILL MISSING** (crate absent).
- **G3.5 `frf-cli`**: **STILL MISSING** (crate absent).
- G3.2 FFI transport (Swift/Kotlin/Dart), G3.3 reconnection, G3.4 bind 5 proto
  services, G3.6 un-fork C# proto: OPEN per phase-15.

### G4 — Operability (OPEN)

- **G4.1 `/readyz`, G4.2 `/metrics`**: **OPEN** — neither route exists in the gateway.
- G4.3 graceful shutdown, G4.4 semantic config validation, G4.5 Keto migration: OPEN.

### G5 — Docs (OPEN)

- **G5.5 LICENSE file**: **OPEN** — no LICENSE file (MIT declared in Cargo.toml).
- G5.1 env reference/`.env.example`, G5.2 runbook, G5.3 security model, G5.4 stale
  README: OPEN per phase-15.

---

## Build Health

- **build check: PASS** — `cargo check --workspace` = exit 0.
- **known violations**: ~50 library `unwrap()`/`expect()` (frf-crdt 36, frf-store-redb
  16) that WILL fail CI once G1.3 turns on `unwrap_used` — this is intended: enable the
  gate, then fix what it flags.
- **test coverage**: not re-measured this pass; prior phases carry unit + E2E suites.

---

## Constraint Check

- **CLAUDE.md violations**: The documented model (per-plane Cargo-feature composition;
  admin-ui served by gateway; zero library unwrap; Keto per-event check with subscribe-
  time caching) diverges from actual code in several places — these divergences ARE the
  phase-16 goals, so they are tracked, not new violations.
- **constraints.md**: not present (N/A).

---

## Goal Progress

| Goal | Status | Reason |
|------|--------|--------|
| G1 Security boundary | NOT MET (open, prioritized) | All 7 sub-items verified open; bypass spans 3 compose files |
| G2 Planes real | NOT MET | admin-UI transport, str0m, federation all unwired |
| G3 Deliverables | NOT MET | frf-sdk-rust + frf-cli still absent |
| G4 Operability | NOT MET | no /readyz, /metrics, graceful shutdown |
| G5 Docs | NOT MET | no LICENSE, runbook, env ref, security model |

---

## Open Questions for Plan Stage

1. **compose split (G1.1):** Confirm the intended contract — keep `DEV_NO_AUTH` +
   `dev-endpoints` in `compose.ci.yml`/`compose.override.yml` (dev/CI only), strip from
   `compose.yml`, and make `dev_no_auth()` compile-gated so it cannot exist in a default
   release build. Should the release image build with no features by default (it already
   does per the Dockerfile) and CI explicitly opt in?
2. **str0m (G2.1):** Implement real WebRTC now, or gate `SFU_MODE=sovereign` off and
   ship LiveKit-hosted as the only supported media path for v1? (Large effort divergence.)
3. **Scope of v1 federation (G2.4/G2.5):** Are Matrix/ATProto bridges in-scope for the
   production release, or deferred (mark unsupported) to keep phase-16 tractable?
4. **`unwrap_used` rollout (G1.3):** Enforce workspace-wide at once (then fix ~50
   sites), or per-crate incrementally? Recommend: enable at workspace level, fix
   frf-crdt/redb, allow-list any truly-unreachable sites with justification.

These are load-bearing scope calls — surface to the operator before planning G2/G3.

---

## Handoff to Plan

Start the change list with **G1** in the order listed (G1.1 compose bypass is the single
most important fix; every other goal assumes the auth boundary holds). Treat G2.1 (str0m)
and G2.5 (federation) as scope decisions requiring operator input before they enter the
change list. Baseline is clean-compiling, so changes can be verified incrementally.

ASSESSMENT COMPLETE
