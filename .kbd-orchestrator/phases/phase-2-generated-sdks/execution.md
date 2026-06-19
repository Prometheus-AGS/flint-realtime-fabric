# Execution: Phase 2 — Generated SDKs

> Phase: phase-2-generated-sdks
> Backend: openspec (claude-code)
> Started: 2026-06-18
> Changes total: 10

## Backend Selection

OpenSpec is available and all changes have proposal.md + tasks.md in
`openspec/changes/`. All changes are executed by claude-code (this session)
in dependency order.

## Dispatch Contract

Changes execute sequentially following dependency order from plan.md:

```
p2-c001 (security fix, unblocked)
p2-c002 (proto annotation, unblocked — parallel with c001)
p2-c003 (buf codegen, after c002)
p2-c004 (Go SDK, after c003)
p2-c005 (TS SDK, after c003)
p2-c006 (C# SDK, after c002)
p2-c007 (gateway tonic, after c001)
p2-c008 (entity-management, after c005)
p2-c009 (admin-ui, after c008)
p2-c010 (E2E smoke, after c004+c005+c006+c007+c009)
```

## QA Gate

- Changes with <3 files modified: skip artifact-refiner; verify manually
- Changes with ≥3 files: `cargo clippy` + `cargo test` or `pnpm typecheck` as applicable
- No `--skip-qa` passed

## Status

| Change | Status |
|--------|--------|
| p2-c001-publish-authz-fix | PENDING |
| p2-c002-proto-csharp-namespace | PENDING |
| p2-c003-buf-config | PENDING |
| p2-c004-sdk-go | PENDING |
| p2-c005-sdk-ts | PENDING |
| p2-c006-sdk-csharp | PENDING |
| p2-c007-gateway-tonic-service | PENDING |
| p2-c008-entity-management-adapter | PENDING |
| p2-c009-admin-ui-scaffold | PENDING |
| p2-c010-e2e-smoke | PENDING |
