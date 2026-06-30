# Goals — phase-15-stage10-live-run-and-retries

> Seeded from: phase-14-stage10-dind-live-triage reflection
> Authored: 2026-06-30

---

## G1 — Execute Stage 10 live in a DinD-capable environment

Run `ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts` with
the Phase 14 fixes in place. All automatable runtime blockers are resolved:

- flint-gate replaces Oathkeeper in compose (c001)
- `DEV_NO_AUTH=true` bypasses JWT verification in `dev-endpoints` builds (c002)
- `tenant_id` present in Phase 6 inject POST bodies (c003)
- "404 in release" test gated behind `DEV_ENDPOINTS_ENABLED=true` (c004)
- Phase 6 Layer 3 tests skipped unless `ENABLE_FEDERATION_STAGE=true` (c005)

Capture full Stage 10 output. Triage any remaining runtime failures by category.

Entry condition: Docker-capable CI (`runs-on: ubuntu-latest --privileged`) or
local Docker Desktop with `/var/run/docker.sock` accessible to the Dagger runner.

## G2 — Commit `.wasm-size-baseline`

After a successful Stage 6 run (WASM build):

```bash
make baseline-wasm
git add .wasm-size-baseline
git commit -m "chore: arm WASM size regression guard"
```

This arms the WASM size regression check in Stage 7.

## G3 — Triage and fix any new Stage 10 runtime failures

For each failure category surfaced in G1:
- One targeted change per category
- Prefer skipping / stubbing over deleting tests pending future infrastructure
- Increase poll timeouts rather than disabling infra tests

Known possible failure categories (pre-live-run hypotheses):
- flint-gate Dockerfile build failure (first compose build of flint-gate)
- CDC timing: replication slot not active before CDC smoke tests poll
- iggy healthcheck window: broker not ready before gateway starts
- Phase 4/5 WebSocket subscribe auth (DEV_NO_AUTH covers publish; subscribe path needs verification)

## G4 — Evaluate and wire Playwright `--retries=2` in Stage 10

After observing flakiness patterns from the G1 live run:
- If ≥1 test category shows intermittent failure: add `--retries=2` to Stage 10
  Playwright invocation in `dagger/codegen.ts`
- If no flakiness: skip — adding retries without evidence is premature

## G5 — Verify flint-gate Dockerfile builds cleanly in compose context

The Phase 14 compose stack references the flint-gate repo at
`/Users/gqadonis/Projects/prometheus/flint-gate`. The Dockerfile has not been
tested in the DinD compose build yet. Verify it:
- Builds successfully from the Dockerfile in the flint-gate repo
- Starts and reaches healthy state (proxy port 4456, admin port 4457)
- Admin endpoint `GET /signing-keys` returns valid JWKS
- Gateway successfully loads signing keys on startup

If the Dockerfile needs adjustment, fix it in the flint-gate repo (not in FRF).
