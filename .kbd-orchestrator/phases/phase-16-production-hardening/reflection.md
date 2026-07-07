# Reflection — phase-16-production-hardening

> Generated: 2026-07-06
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **26 / 26 DONE**. Source assessment: phase-15 production-readiness audit
> (7 CRITICAL, 15 HIGH, 24 MEDIUM, 15 LOW → verdict "NOT production-ready").

## Summary

The phase closed the release-blocking gaps in the goals' dependency order —
security boundary first, then honestly reconciling the advertised planes, then the
missing SDK/CLI deliverables, then operability, then docs. Every goal was grounded
back against the tree during this reflection (not just against the change list): the
compose bypass is gone, the three deliverable crates exist, the docs/release artifacts
are all present, the unwrap gate and readyz/metrics routes are wired, and CI enforces
the gates.

The most important quality of this phase is **honesty about the media/federation
planes (G2)**. Rather than fake WebRTC/federation implementations, those were gated off
or explicitly labeled unimplemented, and the deferrals are recorded in the CHANGELOG's
"Deferred to future phases" section — not silently missing. Several audit findings were
also re-verified during implementation and found overstated (Cedar allow-all, Keto
cache poisoning, str0m "fully broken"); those were documented truthfully instead of
"fixed."

## Goal Achievement

| Goal | Title | Verdict | Evidence |
|------|-------|---------|----------|
| **G1** | Close the security boundary | **MET** | `compose.yml` has no `dev-endpoints`/`DEV_NO_AUTH` (release image built without the feature; bypass code path absent). App-layer tenant-equality guard on publish/subscribe. `unwrap_used`/`expect_used = "deny"` at workspace level + `clippy.toml` allow-in-tests. JWT `iss` verified. Rate-limit + body-limit + CORS layered. flint-gate secret externalized. Cedar surfaces eval errors (no silent allow/deny). |
| **G2** | Make advertised planes real, or remove them | **PARTIAL (by design)** | Admin-UI transport is genuinely end-to-end: gRPC-web (`tonic_web::GrpcWebLayer` + `accept_http1`), real `/ws/v1/signal` route, `rust-embed` admin-ui fallback, port reconciliation. str0m sovereign SFU **gated off** (`SFU_MODE=hosted`) and labeled unimplemented; Matrix inbound / ATProto outbound and LiveKit cross-node relay **explicitly deferred**. Exit criterion ("functions OR honestly labeled") is met; full impls are future work. |
| **G3** | Ship the missing deliverables | **MET** | `frf-sdk-rust` (reconnect/backoff + replay-from-offset), `frf-cli` (`frf`: Keto seed/revoke, broker checkpoint, CDC status), `frf-ffi` transport for Swift/Kotlin via UniFFI. Spine/Signal/Sync/Agent gRPC servers registered; Sync/Agent/Signal clients bound. C# no longer forks proto — generates from frozen `proto-v1`. |
| **G4** | Operability | **MET** | `/readyz` (Keto/JWKS/Iggy probes) + `/metrics` (Prometheus) routes; graceful shutdown on SIGTERM/SIGINT draining in-flight requests + WS streams; semantic config validation at boot; Keto migration step in compose. |
| **G5** | Documentation & release artifacts | **MET** | `docs/ENVIRONMENT.md`, `docs/RUNBOOK.md`, `docs/SECURITY.md`, `.env.example`, README phase status. `LICENSE` (MIT, matches Cargo.toml), `CONTRIBUTING.md`, root `SECURITY.md`, `CHANGELOG.md`, `docs/API-REFERENCE.md` (all 6 services), `docs/decisions/adr-003-ffi-codegen-versions.md`. |

**Score: 4 MET, 1 PARTIAL-by-design (G2).** No goal is NOT-MET. Every CRITICAL (7)
and HIGH (15) is closed or honestly deferred with rationale — the phase completion
criterion.

## Delivered Changes (26)

- **Security (G1):** c001 compose bypass removal, c002 tenant-equality guard, c003
  (str0m transport groundwork), c004/c005 gateway security layers, c006 iss verify,
  c007 Cedar error surfacing, plus the `unwrap_used` workspace gate.
- **Planes (G2):** c008 admin-UI transport (split: transport first, then deferred
  tasks), c009/c010 signal + embed, federation identifier config.
- **Deliverables (G3):** c011 `frf-sdk-rust`, c012 reconnection/backoff, c013 service
  binding (Sync + existing clients), c014 FFI CRDT bridge (Dart→uniffi-bindgen-dart),
  c015 `frf-cli`, c016 C# proto de-fork.
- **Operability (G4):** c017–c021 readyz/metrics/shutdown/config-validation/keto-migrate.
- **Docs (G5):** c022 ENVIRONMENT, c023 RUNBOOK, c024 SECURITY model, c025 README,
  c026 LICENSE/CONTRIBUTING/SECURITY/CHANGELOG/API-REFERENCE/ADR-003.

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with artifact-refiner QA | 0 / 26 |
| First-pass pass rate | n/a |

**No artifact-refiner QA gate was run** (`.refiner/artifacts/` is empty). Quality was
instead enforced inline via the Rust gates that CI runs: `cargo fmt --check`, `clippy
--workspace --lib --bins -- -D warnings -W clippy::pedantic` (plus a `dev-endpoints`
pass and a tests pass), and the `unwrap_used`/`expect_used` denial. This is a gap in
*process* (the KBD QA step was skipped), not necessarily in *output* — but it means the
constraint-violation aggregation this report is meant to summarize does not exist.
**Recommendation:** wire artifact-refiner into `/kbd-execute` for the next phase, or run
it retroactively against this phase's changes before tagging a release.

## Technical Debt Introduced / Carried

1. **Deferred planes (documented, not hidden):** str0m sovereign SFU (real WebRTC),
   Matrix inbound + ATProto outbound protocol impls, `EntityService`/`AuthzService`
   gateway servers, Dart transport bindings, LiveKit cross-node inbound relay. All in
   the CHANGELOG "Deferred" section. Each is a real feature the surface previously
   *implied* existed; hosted/single-node paths work, the sovereign/federated paths do
   not yet.
2. **Dart SDK resolves but exposes no generated API** until `sdks/dart/build_dart.sh`
   runs in an env with `uniffi-bindgen-dart` installed (ADR-003).
3. **QA gate skipped** (see above).
4. **A production-readiness re-audit has not been run.** The phase completion criterion
   asks for a re-run returning "no CRITICAL + materially reduced HIGH." We have strong
   per-goal evidence but not a fresh independent audit pass.

## Lessons Captured

- **Audit findings drift optimistic-pessimistic both ways.** Several CRITICAL/HIGH
  items were overstated (Cedar, Keto cache, str0m). Re-verifying each during
  implementation — and documenting the correction — was higher-value than blindly
  "fixing" a misdiagnosed defect.
- **"Healthy but does nothing" is worse than "honestly disabled."** The G2 reframing
  (implement OR label) let the phase converge without faking media/federation.
- **Framework conflicts surface late.** flutter_rust_bridge cannot bridge a UniFFI
  crate (it panics on `#[uniffi::export]` and injects `mod frb_generated`). Discovered
  mid-c014; resolved by unifying on UniFFI across all mobile SDKs (ADR-003). Pin FFI
  toolchains in an ADR *before* scaffolding bindings next time.
- **The pipeline-enforce hook reads `progress.json`, not intent.** Bundling the
  progress bump and a `reflect`-referencing string in one command tripped the gate at
  25/26. Bump progress first, in a command with no next-stage reference.

## Recommended Next Phase

**phase-17-plane-completion-and-release-audit** (or split):

1. **Re-run the production-readiness audit** independently and confirm zero CRITICAL +
   reduced HIGH against real code — close the phase-16 completion criterion properly.
2. **Wire artifact-refiner QA** into execute so this reflection's quality table is real
   next time.
3. **Complete one deferred plane at a time**, highest-value first: likely
   `EntityService`/`AuthzService` gateway servers (pure Rust, no external SFU/federation
   dependency) before str0m WebRTC and the federation protocol impls.
4. **Finish Dart transport bindings** (`uniffi-bindgen-dart`) so the mobile SDK story
   is complete across Swift/Kotlin/Dart.

Do **not** advertise a plane as shipped until its re-audit finding is closed.
