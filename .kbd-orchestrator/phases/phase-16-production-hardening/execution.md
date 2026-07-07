# Execution — phase-16-production-hardening

> Authored: 2026-07-06
> Backend: **openspec** (driven via `/kbd-apply`, one change at a time)

## Backend selection

**Chosen: `openspec`.**

- `project.json → change_backend = "openspec"`.
- All 26 changes are already OpenSpec structures under `openspec/changes/p16-c*/`
  (each with `proposal.md` + `tasks.md`).
- Spec-backed traceability is required for a production-hardening / security phase —
  every change maps to a phase-15 finding ID and a G-goal.

**Rejected alternatives:** `native-tool` (no external tool has a better view of this
Rust workspace than direct apply); `hybrid` (decomposition already done in plan.md);
`manual` (all changes are automatable code/config/docs).

## Dispatch contract

Task execution is driven **exclusively through `/kbd-apply <change-id>`**, NOT bare
`/opsx:apply`. `/kbd-apply` wraps the OpenSpec backend and drives ONE task at a time so
KBD stays the source of truth: every task boundary fires KBD `task:before`/`task:after`
hooks, emits the plain-text position signal, and syncs `progress.json` + the waypoint.
Bare `/opsx:apply` is unmodified upstream OpenSpec (no hooks, no progress sync) and must
not be used to drive tasks.

### Apply order (security-first; G1 gates G2+)

```
G1  p16-c001 → c002 → c003 → c004 → c005 → c006 → c007   ← must fully complete first
G2  p16-c008 → c009 → c010
G3  p16-c011 → c012 → c013 → c014 → c015 → c016
G4  p16-c017 → c018 → c019 → c020 → c021
G5  p16-c022 → c023 → c024 → c025 → c026
```

**Hard gate:** do not begin any G2 change until all seven G1 changes are DONE and
archived. Every downstream change assumes the auth boundary holds.

### Dependency notes
- `p16-c012`, `p16-c014` depend-on `p16-c011` (frf-sdk-rust must exist first).
- `p16-c025` depends-on `p16-c011` + `p16-c015` (README references the new crates).

## Per-change lifecycle

For each change, in order:

1. `/kbd-apply <change-id>` — walks that change's `tasks.md` one task at a time,
   firing per-task hooks and syncing progress.
2. On all tasks `[x]`: change status → DONE in `progress.json`.
3. **QA gate (artifact-refiner):** run `/refine-validate "<change-id>"` unless the
   change qualifies to skip (< 3 files modified, or docs-only). Most G5 changes and
   several small G1/G4 changes skip; multi-file code changes (c001, c003, c008, c011,
   c013, c014, c015) get the gate.
4. PASS → `/opsx:verify <change-id>` → `/opsx:archive <change-id>`.
   FAIL → mark change BLOCKED in `progress.json`, run `/refine-code "<change-id>"`.

## Recommended agent per change

Per `plan.md`: rust-reviewer for gateway/adapter code, tdd-guide for the tenant-equality
guard (c002), security-reviewer for auth/secret/security-model changes (c004, c006, c024),
code-architect for new crates + transport wiring (c008, c011, c014, c015), csharp-reviewer
for c016, doc-updater for G5 docs, devops-engineer for ops/runbook (c021, c023).

## Verification per change

- Rust changes: `cargo check --workspace` + `cargo clippy --workspace -- -D warnings
  -W clippy::pedantic` (and `-W clippy::unwrap_used` once c003 lands) + relevant
  `cargo test`.
- Config/compose changes: build the affected image / run the affected compose path.
- Docs changes: link/consistency check only.

## First pending change

**`p16-c001` — Remove auth bypass from production compose.** Strip
`CARGO_FEATURES: dev-endpoints` + `DEV_NO_AUTH` from `compose.yml`, keep them in
`compose.ci.yml`/`compose.override.yml`, and compile-gate `dev_no_auth()` so a default
release build cannot reach the bypass. Verify by building the default (no-feature)
image and asserting publish/subscribe require a token.

## Status

Execution-ready. Dispatch has NOT auto-run — `/kbd-execute` writes the contract; the
operator (or `/kbd-apply`) drives task execution. Awaiting `/kbd-apply p16-c001`.
