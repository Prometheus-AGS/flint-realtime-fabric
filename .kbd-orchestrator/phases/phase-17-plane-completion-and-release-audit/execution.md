# Execution — phase-17-plane-completion-and-release-audit

> Written by `/kbd-execute`. Task-walking is owned by `/kbd-apply` (one change,
> one task at a time). This file is the dispatch contract, not a task runner.

## Backend

**OpenSpec.** Detected via `.kbd-orchestrator/project.json` (`change_backend: openspec`)
and the `openspec/` directory. All 10 changes are scaffolded under
`openspec/changes/p17-cNNN/` with `proposal.md` (Status: PROPOSED) + `tasks.md`.

Rationale: this phase is spec-backed traceability against a frozen proto contract and a
clean-architecture dependency rule — OpenSpec's per-change proposal/tasks/verify/archive
loop is the right fit. No native tool offers better decomposition here.

## Dispatch contract

Drive each change through `/kbd-apply p17-cNNN` — **never** bare `/opsx:apply`. The apply
driver fires the per-task hooks, syncs `progress.json` + the waypoint, and emits the
plain-text `Starting/Completed task i of n` signals.

Per change, the loop is:
1. `/kbd-apply p17-cNNN` — walk its tasks one per turn (implement, then `end-task`).
2. On the final task, the QA gate runs (see below), then `verify` + `archive`.
3. Bump `progress.json` (`changes_completed++`, move id → `completed_changes`), refresh
   the waypoint, `sed` the proposal PROPOSED → DONE.

## Ordered changes (10)

| Order | Change | Recommended agent | QA-gated? |
|------|--------|-------------------|-----------|
| 1 | p17-c001 — compose.override footgun | security-reviewer | no (constraints.md not authored until c003; <3 prod files) |
| 2 | p17-c002 — enforce JWT_ISSUER in prod | rust-reviewer | no (pre-c003) |
| 3 | **p17-c003 — author constraints.md + wire /refine-validate** | general-purpose | n/a (this IS the gate) |
| 4 | p17-c004 — EntityService gateway server | rust-reviewer | **yes** |
| 5 | p17-c005 — AuthzService gateway server | rust-reviewer | **yes** |
| 6 | p17-c006 — SDK/FFI service + resilience parity | rust-reviewer | **yes** |
| 7 | p17-c007 — CLI CDC slot mgmt + broker-offset inspect | rust-reviewer | **yes** |
| 8 | p17-c008 — finish Dart bindings | dart-build-resolver | **yes** |
| 9 | p17-c009 — docs: API-reference + str0m doc drift | doc-updater | skip (docs-only) |
| 10 | p17-c010 — re-audit + release sign-off | code-reviewer | skip (verification change) |

## Per-change QA gate

**Wired by c003.** The gate is a runnable helper — `.kbd-orchestrator/bin/qa-gate.sh
<change-id> --kind <rust|config|docs|frontend|tooling>` — that evaluates the **BLOCKING**
subset of `.kbd-orchestrator/constraints.md` (compiles, clippy pedantic + `unwrap_used`,
fmt, ≤500-line files, no hardcoded secret, valid OpenSpec delta; plus a no-`any` check for
frontend) and writes `.refiner/artifacts/<change-id>/refinement_log.md`. Exit 0 = ALL
PASS; exit 1 = a BLOCKING constraint failed.

**Ordering dependency:** the gate reads `constraints.md`, authored by **c003** — so it
could not run for c001/c002 (they ran before it, and were config-only). Documented so the
skip is not silent.

- **c001, c002** → ran *before* the gate existed (config-only). Skip is intentional.
- **c003** → *is* the gate wiring (constraints.md + qa-gate.sh + this contract). No self-QA.
- **c004–c008** → **QA gate ON**: after the change reaches DONE, run
  `.kbd-orchestrator/bin/qa-gate.sh p17-cNNN --kind rust` (c008 uses `--kind frontend` for
  the Dart/TS surface). **ALL PASS** → `openspec validate` → `openspec archive`. **ANY
  FAIL** → mark BLOCKED in `progress.json`, fix, re-run the gate.
- **c009 (docs-only), c010 (verification change)** → QA skipped per the documented skip
  rules (documentation-only / verification). c010 instead re-runs the full gate suite on a
  clean checkout as its own deliverable.

This closes phase-16's process debt: from c004 onward, every code change is QA-gated
against `constraints.md` — the gate is no longer silently skipped, and it produces a
persisted `refinement_log.md` per change.

### FAIL path (BLOCKED → refine → re-run)

When `qa-gate.sh` exits non-zero (a BLOCKING constraint failed):

1. **Mark BLOCKED.** In the phase `progress.json`, set the change's state to `BLOCKED`
   (do **not** advance `changes_completed`, do **not** archive). Record which constraint
   failed from the change's `.refiner/artifacts/<id>/refinement_log.md`.
2. **Refine.** Fix the specific violation (e.g. split a >500-line file, remove a library
   `unwrap`, add the missing spec delta). Keep the fix scoped to the failing constraint.
3. **Re-run the gate.** `.kbd-orchestrator/bin/qa-gate.sh <id> --kind <kind>`. On ALL
   PASS, clear the BLOCKED state and proceed to `openspec validate` → `openspec archive`.
4. **Never archive a change with a red gate.** Archiving is allowed only after the gate's
   most recent run for that change is ALL PASS. A change may cycle BLOCKED→refine→re-run
   any number of times; only the green result unlocks archive.

## Rust quality gates (every code change, CI-equivalent, run locally before end-task)

```
cargo fmt --check --all
cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used
cargo clippy --workspace --tests -- -D warnings -W clippy::pedantic
cargo check --workspace
```

Plus the standing rules: no file >500 lines, `#[non_exhaustive]` on public enums, newtype
IDs, tracing spans across port boundaries, no adapter imports in domain/app, never log
JWTs/tuples/tenant secrets.

## Scope guard (deferred — do NOT implement this phase)

str0m real WebRTC, Matrix inbound / ATProto outbound / LiveKit cross-node inbound, and the
admin-ui OIDC login flow are **deferred to a future phase-18**. c009/c010 re-affirm these
deferrals with rationale and keep the docs honest — but no change here implements them.

## First action

`/kbd-apply p17-c001`
