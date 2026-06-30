# Goals — phase-14-stage10-dind-live-triage

> Seeded from: phase-13-live-layer3-e2e-validation reflection
> Authored: 2026-06-30

---

## G1 — Execute Stage 10 live in a DinD-capable environment

Run `ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts` in a
privileged Docker host (GitHub Actions `runs-on: ubuntu-latest` with
`--privileged`, or a local Docker Desktop environment). Capture the full
Stage 10 output and triage all failures by category.

Expected failure categories (from Phase 13 assessment):
- **Auth** — Kratos/Oathkeeper not reachable or JWT misconfigured
- **CDC timing** — replication slot not active before CDC smoke tests run
- **Tuwunel/federation** — `inject-federation-event` receives 200 but
  downstream Matrix bridge is absent (no Tuwunel in compose)
- **Playwright assertion** — flaky timing in E2E specs

## G2 — Commit `.wasm-size-baseline`

After the first successful Stage 6 run (which may precede Stage 10 completion):

```bash
make baseline-wasm
```

Commit `.wasm-size-baseline` to arm the WASM size regression guard.

## G3 — Fix Stage 10 runtime failures found in G1

Resolve each concrete failure category identified in G1. For each fix:
- One targeted change per failure category
- Prefer skipping / stubbing over deleting for federation tests pending Tuwunel
- Prefer increasing poll timeouts over disabling CDC tests

## G4 — Gate federation E2E behind `ENABLE_FEDERATION_STAGE` (if Tuwunel absent)

If the Matrix bridge (Tuwunel) is not available in the CI compose stack, add a
separate `ENABLE_FEDERATION_STAGE` environment variable to gate the Phase 6
federation smoke tests. This avoids blocking the rest of Stage 10 on an
undeployable dependency.

## G5 — Add Stage 10 `--retries=2` and `--shard` where beneficial

Evaluate whether flaky infrastructure tests benefit from Playwright `--retries`
or test sharding. Wire any configuration into `dagger/codegen.ts` Stage 10.
