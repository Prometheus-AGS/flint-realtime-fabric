# Execution — phase-18-media-federation-and-auth-flow

> Written by `/kbd-execute`. Task-walking is owned by `/kbd-apply` (one change, one task
> at a time). This is the dispatch contract, not a task runner.

## Backend

**OpenSpec.** Detected via `.kbd-orchestrator/project.json` (`change_backend: openspec`).
All 10 changes scaffolded under `openspec/changes/p18-cNNN/` with `proposal.md`
(Status: PROPOSED) + `tasks.md`.

## Dispatch contract

Drive each change through `/kbd-apply p18-cNNN` — **never** bare `/opsx:apply`. Per change:
1. `/kbd-apply p18-cNNN` walks its tasks one per turn (implement, then `end-task`).
2. On the final task, run the QA gate (below), then add a `specs/` delta, `openspec
   validate`, `openspec archive --yes`.
3. Bump `progress.json` (`changes_completed++`, id → `completed_changes`), refresh the
   waypoint, `sed` the proposal PROPOSED → DONE.

## Ordered changes (10)

| Order | Change | Recommended agent | QA-gated? |
|------|--------|-------------------|-----------|
| 1 | **p18-c001 — str0m routing bug (HIGH, unblocks signaling)** | rust-reviewer | **yes** |
| 2 | p18-c002 — FEDERATION_CHANNEL_ID guard | rust-reviewer | **yes** |
| 3 | p18-c003 — sfu_mode wire consistency | rust-reviewer | **yes** |
| 4 | p18-c004 — Dart doc drift | doc-updater | skip (docs-only) |
| 5 | p18-c005 — admin-ui token-flow hardening | typescript-reviewer | **yes** (`--kind frontend`) |
| 6 | p18-c006 — str0m Rtc round-trip spike | rust-reviewer | **yes** |
| 7 | p18-c007 — Matrix inbound /sync loop | rust-reviewer | **yes** |
| 8 | p18-c008 — ATProto outbound PDS write | rust-reviewer | **yes** |
| 9 | p18-c009 — Dart async-transport shim | dart-build-resolver | **yes** (`--kind frontend`) |
| 10 | p18-c010 — docs + re-audit sign-off | code-reviewer | skip (verification) |

## Per-change QA gate (carried from phase-17)

The gate — `.kbd-orchestrator/bin/qa-gate.sh <change-id> --kind <rust|frontend|docs>` —
already exists (authored p17-c003) and evaluates the BLOCKING subset of
`.kbd-orchestrator/constraints.md`, writing `.refiner/artifacts/<id>/refinement_log.md`.

- **c001–c003, c006–c008** → `--kind rust`. **c005, c009** → `--kind frontend` (TS / Dart
  surface).
- **ALL PASS** → `openspec validate` → `openspec archive`. **ANY FAIL** → mark BLOCKED in
  `progress.json`, fix, re-run the gate.
- **c004 (docs-only), c010 (verification)** → QA skipped per the documented skip rules;
  c010 instead re-runs the full gate suite on a clean checkout as its deliverable.

## Rust quality gates (every code change, run locally before end-task)

```
cargo fmt --check --all
cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used
cargo clippy --workspace --tests -- -D warnings -W clippy::pedantic
cargo check --workspace
```
Plus: no file >500 lines, `#[non_exhaustive]` on public enums, newtype IDs, tracing spans
across port boundaries, no adapter imports in domain/app, never log JWTs/tuples/tenant.

Frontend (c005/c009): `cd admin-ui && pnpm typecheck && pnpm lint` (no `any`); Dart:
`dart analyze` (hand-authored clean; generated code analyzer-excluded).

## Scope guard (deferred — do NOT implement this phase)

**Full str0m sovereign SFU** (per-session Rtc + UDP/ICE/DTLS loop + RTP fan-out),
**LiveKit cross-node inbound relay**, and the **full admin-ui OIDC login flow** (blocked
on an IdP decision) are deferred to **phase-19**. c006 is a spike only (round-trip or
finding, gate stays off unless media flows); c005 is token-hardening, not OIDC; c010
re-affirms these deferrals. No change here implements them.

## First action

`/kbd-apply p18-c001`
