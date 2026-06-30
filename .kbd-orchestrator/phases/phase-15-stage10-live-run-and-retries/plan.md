# Plan — phase-15-stage10-live-run-and-retries

> Authored: 2026-06-30
> Backend: openspec
> Assessment: assessment.md

---

## Overview

One automatable fix must land before Stage 10 can succeed. All other changes
in this phase are conditional on the live run output or are manual operator
steps. The plan is ordered as: fix the blocker → trigger the live run →
triage failures → wire retries if needed → commit WASM baseline.

---

## Change Order

### p15-c001 — Fix Dagger Stage 10 port references (EXECUTE NOW)

**Priority**: CRITICAL — blocks everything else  
**Backend**: openspec  
**Agent**: Claude Code (direct edit, trivial two-line change)  
**Files**: `dagger/codegen.ts`

The Stage 10 Dagger block polls `localhost:8080` and sets
`GATEWAY_URL=http://localhost:8080`. The gateway is host-mapped at port
`28080` in compose.yml. Both references must change to `28080`.

Change spec: `openspec/changes/p15-c001-dagger-port-fix/`

---

### p15-c003 — Run Stage 10 live (MANUAL — operator action)

**Priority**: HIGH — unblocked after p15-c001  
**Backend**: manual  
**Agent**: operator (requires DinD-capable environment)

After p15-c001 is committed:

```bash
ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```

Capture full output. Feed failure output back to `/kbd-execute` for G3 triage.

Entry condition: Docker Desktop with `/var/run/docker.sock` accessible to the
Dagger runner, OR GitHub Actions with `--privileged`.

---

### p15-c002 — Fix flint-gate Dockerfile (CONDITIONAL on p15-c003)

**Priority**: HIGH if live run fails at compose build step  
**Backend**: openspec (if needed; creates change in flint-gate repo)  
**Agent**: Claude Code  
**Files**: `/Users/gqadonis/Projects/prometheus/flint-gate/Dockerfile`

Only create this change if `docker compose up -d` fails during the Stage 10
live run. Check for:
- `Cargo.lock` missing in flint-gate repo (fix: add `Cargo.lock` or use
  `COPY Cargo.toml ./` without lock)
- Binary output name mismatch (fix: use correct binary name in `COPY --from=builder`)

If live run succeeds, skip this change entirely.

---

### p15-c004 — Wire Playwright `--retries=2` (CONDITIONAL on p15-c003)

**Priority**: MEDIUM — if ≥1 test shows intermittent failure  
**Backend**: openspec  
**Agent**: Claude Code  
**Files**: `dagger/codegen.ts`

After the live run output: if any test category shows intermittent failure
(same test passes on re-run), add `"--retries=2"` to the Stage 10 Playwright
invocation array in `dagger/codegen.ts`.

If no flakiness: skip this change.

---

### G2 — Commit `.wasm-size-baseline` (MANUAL — after Stage 6 completes)

**Priority**: MEDIUM  
**Backend**: manual  
**Agent**: operator

After a successful Stage 6 WASM build during p15-c003:

```bash
make baseline-wasm
git add .wasm-size-baseline
git commit -m "chore: arm WASM size regression guard"
```

This arms the WASM size regression check in Stage 7 for future runs.

---

## Summary

| Change | Type | Trigger | Files |
|--------|------|---------|-------|
| p15-c001 | automatable | immediate | `dagger/codegen.ts` |
| p15-c003 | manual (operator) | after p15-c001 | — |
| p15-c002 | conditional | if compose build fails | `flint-gate/Dockerfile` |
| p15-c004 | conditional | if flakiness observed | `dagger/codegen.ts` |
| G2 | manual (operator) | after Stage 6 succeeds | `.wasm-size-baseline` |

Total automatable changes (definite): **1**  
Total conditional changes: **2**  
Total manual operator steps: **2**

---

## Execution Notes

- Execute p15-c001 immediately in this session.
- p15-c003 (live run) requires a separate DinD-capable terminal or CI job.
- After the live run, re-invoke `/kbd-execute phase-15-stage10-live-run-and-retries`
  with the failure output to handle G3 triage and decide c002/c004.
- Do not advance to reflect until the live run has been attempted and all
  conditional changes are resolved or explicitly skipped.
