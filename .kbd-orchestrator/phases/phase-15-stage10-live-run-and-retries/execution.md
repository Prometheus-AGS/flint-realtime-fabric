# Execution — phase-15-stage10-live-run-and-retries

> Authored: 2026-06-30
> Backend: openspec (automatable change) + manual (live run + conditional changes)

---

## Backend Selection

| Change | Backend | Rationale |
|--------|---------|-----------|
| p15-c001 | openspec / direct edit | Trivial 2-line port number correction |
| p15-c003 | manual | DinD live run requires operator action |
| p15-c002 | openspec / conditional | Only if compose build fails in p15-c003 |
| p15-c004 | openspec / conditional | Only if flakiness observed in p15-c003 |
| G2 | manual | Requires WASM build artifact from Stage 6 |

---

## p15-c001 — COMPLETED

**Change**: Fix Dagger Stage 10 port references  
**Files**: `dagger/codegen.ts`  
**Status**: DONE

Two edits applied to the Stage 10 integration block:

1. Healthz poll: `http://localhost:8080/healthz` → `http://localhost:28080/healthz`
2. `GATEWAY_URL`: `http://localhost:8080` → `http://localhost:28080`

Rationale: `compose.yml` maps gateway as `28080:8080`. The Dagger runner in
DinD sees the host-mapped port `28080`. Polling `8080` would time out after
60 seconds, blocking every test in Stage 10.

Verification: `grep localhost:8080 dagger/codegen.ts` → no matches.

---

## p15-c003 — PENDING OPERATOR ACTION

**Trigger**: After p15-c001 commit merges.

Operator runs from a DinD-capable environment:

```bash
ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```

Capture the full output. Feed failure categories back to
`/kbd-execute phase-15-stage10-live-run-and-retries` to trigger c002/c004
and G3 triage.

---

## Conditional Changes (PENDING live run)

### p15-c002 — flint-gate Dockerfile (if compose build fails)

If `docker compose up -d` fails at the flint-gate build step:
1. Check `Cargo.lock` is present in `/Users/gqadonis/Projects/prometheus/flint-gate`
2. Check binary name in release output matches `flint-gate`
3. Fix Dockerfile accordingly

### p15-c004 — Playwright `--retries=2` (if flakiness observed)

If ≥1 test category shows intermittent failure across the live run:
- Add `"--retries=2"` to the `pnpm exec playwright test` args array in
  `dagger/codegen.ts` Stage 10 block

---

## Dispatch Contract

```
p15-c001  →  DONE (committed)
p15-c003  →  WAITING_FOR_OPERATOR
p15-c002  →  CONDITIONAL (on p15-c003 compose build failure)
p15-c004  →  CONDITIONAL (on p15-c003 flakiness data)
G2        →  WAITING_FOR_OPERATOR (after Stage 6 build)
```

Next action: commit p15-c001 → operator triggers Stage 10 live run →
feed output back to `/kbd-execute` for G3 triage + conditional changes.
