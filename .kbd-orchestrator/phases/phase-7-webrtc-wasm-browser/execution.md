# Execution — Phase 7: WebRTC str0m SFU + WASM SDK Depth

> Started: 2026-06-21 · Backend: openspec · Tool: kbd-execute

## Backend Selection

**openspec** — `openspec/` directory present; `project.json` confirms
`"change_backend": "openspec"`. All 7 changes tracked in
`openspec/changes/p7-c00{1-7}-*/`.

## Dispatch Contract

- p7-c001, p7-c002, p7-c003 run in parallel (no mutual deps)
- p7-c004 is the sync point (requires p7-c001 + p7-c003)
- p7-c005 and p7-c006 run in parallel after p7-c002 + p7-c004
- p7-c007 is the final gate

## Quality Gate Protocol

After each change: `cargo check --workspace` → `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` → change-specific tests.

After p7-c005: additionally `cd admin-ui && fnm exec --using=24 pnpm typecheck && fnm exec --using=24 pnpm build`.
After p7-c007: additionally `SKIP_INTEGRATION=true fnm exec --using=24 pnpm exec playwright test e2e/phase7-smoke.spec.ts`.

## Change Status

| Change | Status | Notes |
|---|---|---|
| p7-c001-frf-media-str0m | PENDING | — |
| p7-c002-frf-wasm-sdk-depth | PENDING | — |
| p7-c003-runagent-bidi-upgrade | PENDING | — |
| p7-c004-gateway-str0m-dev-inject | PENDING | awaits p7-c001 + p7-c003 |
| p7-c005-admin-ui-webrtc-wasm | PENDING | awaits p7-c002 + p7-c004 |
| p7-c006-dagger-ci-node24-wasm-e2e | PENDING | awaits p7-c002 |
| p7-c007-e2e-phase7-smoke | PENDING | awaits p7-c005 + p7-c006 |
