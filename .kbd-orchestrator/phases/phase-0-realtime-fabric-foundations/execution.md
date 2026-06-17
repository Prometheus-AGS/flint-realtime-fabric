# Execution — phase-0-realtime-fabric-foundations

## Backend: openspec (inline)

All changes are implemented directly in this session via the `openspec` change backend. Tasks are walked sequentially per change, with `progress.json` updated at each change boundary.

## Dispatch Contract

- Change driver: inline (Claude Code, this session)
- Task tracking: `openspec/changes/<id>/tasks.md` (checkbox updates)
- Phase tracking: `.kbd-orchestrator/phases/phase-0-realtime-fabric-foundations/progress.json`
- QA gate: artifact-refiner invoked after each change (≥3 files)
- Archive: `/opsx:archive <change-id>` after QA pass

## Change Execution Log

| Change | Status | Notes |
|--------|--------|-------|
| p0-c001-workspace-restructure | IN_PROGRESS | Writing |
| p0-c002-frf-domain | PENDING | |
| p0-c003-frf-ports | PENDING | |
| p0-c004-frf-proto | PENDING | |
| p0-c005-frf-gateway-stub | PENDING | |
| p0-c006-dagger-ci | PENDING | |
