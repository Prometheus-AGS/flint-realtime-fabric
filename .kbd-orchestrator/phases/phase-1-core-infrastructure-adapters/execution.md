# Execution: Phase 1 — Core Infrastructure Adapters

> **RFC-FRF-002 · Prometheus AGS**
> Backend: `openspec`
> Dispatch: `/kbd-apply` → OpenSpec changes
> Started: 2026-06-17

## Backend Selection

**OpenSpec** — all changes have `proposal.md` + `tasks.md` under `openspec/changes/`. Change traceability required.

## Dispatch Contract

- Each change executed via `/kbd-apply <phase> <change-id>`
- Per-change QA gate: artifact-refiner validate → opsx:verify → opsx:archive
- Changes with fewer than 3 files (documentation-only) skip QA
- Waypoint and `progress.json` updated after each completed change

## Change Dispatch Table

| Order | Change ID | Status | Notes |
|-------|-----------|--------|-------|
| 1 | p1-c001-workspace-expansion | IN_PROGRESS | Mandatory first; scaffolding only |
| 2 | p1-c002-frf-app | PENDING | After p1-c001 |
| 3 | p1-c003-frf-broker-iggy | PENDING | Parallel with c004, c005 after c001 |
| 4 | p1-c004-frf-authz-keto | PENDING | Parallel with c003, c005 after c001 |
| 5 | p1-c005-frf-identity-ory | PENDING | Parallel with c003, c004 after c001 |
| 6 | p1-c006-frf-postgres-cdc | PENDING | After c001 |
| 7 | p1-c007-gateway-subscription-mux | PENDING | After c001–c005 complete |

## QA Constraints

From `.kbd-orchestrator/constraints.md` (if absent: use project CLAUDE.md code quality gates):
- `cargo check --workspace` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0  
- `cargo fmt --check --all` exits 0
- No `unwrap()` / `expect()` in library crates
- `#[non_exhaustive]` on all public enums
