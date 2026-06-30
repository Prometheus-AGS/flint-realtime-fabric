# Assessment — Phase 11: Layer 3 Full-Stack E2E + WASM Size Optimization + CDC Wiring

> Written: 2026-06-23 · Tool: kbd-assess

---

## Codebase State at Phase Entry

| Artifact | Status |
|----------|--------|
| `cargo check --workspace` | PASS — zero errors (15.75s, all 15 crates) |
| `frf-postgres-cdc` | COMPILES — `PostgresCdcConsumer` fully implemented, wired into `frf-gateway` via `spawn_cdc_consumer` |
| CDC env vars in `compose.yml` | ABSENT — gateway `environment:` block has no `CDC_ENABLED`, `CDC_REPLICATION_URL`, etc. |
| `deploy/postgres/init.sql` | EXISTS — creates `frf_pub FOR ALL TABLES` publication; logical replication enabled in compose command |
| Layer 2 E2E specs | DONE (phase 10) — 3 files, 18 tests, `SKIP_INTEGRATION` gated |
| Layer 3 E2E specs | EXIST in `p7-smoke.spec.ts`, `phase4-smoke.spec.ts`, `phase5-smoke.spec.ts`, `phase6-smoke.spec.ts` — gated on `WASM_AVAILABLE` or `GATEWAY_URL` |
| `layer2-crdt.spec.ts` | ABSENT |
| `frf-wasm crdt_apply_delta` | EXPORTED via `#[wasm_bindgen]` in `crates/frf-wasm/src/crdt.rs` |
| Dagger Stage 6 `--no-opt` | PRESENT — wasm-opt bypassed due to wasm-bindgen-cli 0.13.1 bundling older wasm-opt |
| wasm-opt on PATH | ABSENT — neither `~/.cargo/bin/wasm-opt` nor system `wasm-opt`; `binaryen` not in homebrew |
| wasm-pack version | 0.13.1 (latest available; ships its own wasm-opt binary internally) |
| Kotlin `tasks.test {}` guard | ABSENT — `lib/build.gradle.kts` has `tasks.test { systemProperty(...) }` but no `enabled = false` |
| `./gradlew :lib:compileKotlin` | PASSES (verified phase 10) |
| Dagger Stage 10 (integration) | WIRED — `ENABLE_INTEGRATION_STAGE=true`; runs compose up + all E2E files |
| Dagger Stage 10 gateway healthz | POLLS `http://localhost:8080/healthz` — but compose ports remapped to 28080; **BUG** |

---

## Gap Analysis by Goal

### G1 — Layer 3 Full-Stack E2E

**Status: PARTIALLY WIRED — needs fixes before CI run**

Present:
- Dagger Stage 10 exists and starts the compose stack then runs all `e2e/` specs
- Layer 3 test blocks exist in `p7-smoke.spec.ts` (WASM transport), `phase4-smoke.spec.ts` (CDC), `phase5-smoke.spec.ts` (agent bus), `phase6-smoke.spec.ts` (federation bus)
- Compose stack validated in phase 10 (all services reach healthy state)

Gaps:
1. **Dagger Stage 10 healthz port bug**: polls `http://localhost:8080/healthz` but compose maps gateway to host port `28080:8080`. Inside the Dagger container the gateway is at container port 8080 (Docker-in-Docker), so this may actually resolve correctly — but the GATEWAY_URL passed to Playwright must be `http://localhost:28080` (the host-mapped port). Currently no `GATEWAY_URL` env var is set in Stage 10 for the `layer2-*.spec.ts` tests.
2. **`GATEWAY_URL` not passed to Playwright in Stage 10**: Layer 2 tests gate on `!process.env["GATEWAY_URL"]` → all Layer 2 tests will skip inside Dagger Stage 10 without this env var set.
3. **CDC Layer 3 test** (`phase4-smoke.spec.ts`) will fail: requires a real Postgres CDC event published through the gateway. This needs `CDC_ENABLED=true` env in compose gateway service, which is absent.
4. **iggy topic creation**: Layer 3 subscribe tests may fail if the iggy stream/topic is not pre-created. `iggy-server` starts fresh; gateway's `ensure_topic` logic depends on broker adaptor startup.

### G2 — wasm-opt Upgrade

**Status: NOT STARTED**

Present:
- `--no-opt` flag in Dagger Stage 6 (added phase 10) prevents WASM binary optimization
- `wasm-pack 0.13.1` bundles its own wasm-opt (version unknown, but older than bulk-memory support)
- No system `wasm-opt` binary available

Gaps:
- Need to identify the wasm-opt version bundled with wasm-pack 0.13.1 and whether upgrading wasm-bindgen-cli resolves the bulk-memory issue
- The `binaryen` package (which provides `wasm-opt`) is the standard install path; version ≥ 116 required for bulk-memory + Loro
- Option A: install `binaryen` in Dagger Stage 6 and invoke `wasm-opt` directly, bypassing wasm-pack's bundled one
- Option B: upgrade wasm-bindgen-cli to a version that bundles wasm-opt ≥ 116

### G3 — PostgreSQL CDC Wiring

**Status: IMPLEMENTATION COMPLETE, COMPOSE INTEGRATION MISSING**

Present:
- `PostgresCdcConsumer` fully implemented and wired into `frf-gateway` via `spawn_cdc_consumer`
- `deploy/postgres/init.sql` creates `frf_pub FOR ALL TABLES`
- Postgres configured with `wal_level=logical`, `max_replication_slots=5`
- `frf_pub` publication ready for subscription
- CDC env config in gateway: `CDC_ENABLED`, `CDC_REPLICATION_URL`, `CDC_SLOT_NAME`, `CDC_PUBLICATION_NAME`, `CDC_TENANT_ID`, `CDC_CHANNEL_PATH`

Gaps:
1. **`compose.yml` gateway environment missing CDC vars** — gateway will start with `cdc_enabled = false` and will not subscribe to WAL
2. **No CDC replication slot pre-creation** — Postgres `frf_pub` is created but no replication slot (`frf_slot`) is created in `init.sql`. `PostgresCdcConsumer::run_until_shutdown` calls `ensure_replication_slot` which will auto-create it, but the Postgres user must have `REPLICATION` privilege
3. **Missing `REPLICATION` privilege grant** in `init.sql` — `CREATE PUBLICATION` requires superuser or `pg_write_all_data`; `REPLICATION` attribute must be on the connecting user for logical replication
4. **CDC tenant/channel config** — requires a real tenant UUID; needs a fixture value in compose for testing

### G4 — CRDT Layer 2 Test

**Status: NOT STARTED**

Present:
- `crdt_apply_delta` is exported via `#[wasm_bindgen]` in `crates/frf-wasm/src/crdt.rs`
- Generated `frf_wasm.js` in `sdks/ts/frf-wasm/` exposes `crdt_apply_delta`
- No `layer2-crdt.spec.ts` file exists

Gaps:
- Need `admin-ui/e2e/layer2-crdt.spec.ts` that:
  - Layer 1: loads `/#demo/signaling`, verifies CRDT Apply Delta button exists and is disabled until input provided
  - Layer 2 (gated on `SKIP_INTEGRATION=false`): imports `frf-wasm` in a page fixture, calls `crdt_apply_delta` with a known snapshot+delta pair, checks result length > 0

### G5 — Kotlin JNI Test Guard

**Status: NOT STARTED**

Present:
- `lib/build.gradle.kts` has `tasks.test { systemProperty("jna.library.path", ...) }` — tests would attempt to load JNI at runtime
- `./gradlew :lib:compileKotlin` passes (verified phase 10)
- No `enabled = false` guard to prevent runtime JNI failures in CI

Gaps:
- Add `tasks.withType<Test> { enabled = false }` to `lib/build.gradle.kts`
- Verify `./gradlew :lib:compileKotlin` still passes (compile-only check)

---

## Phase 11 Exit Criteria

- [ ] Dagger Stage 10 runs Layer 2 tests with `GATEWAY_URL=http://localhost:28080` set
- [ ] `compose.yml` gateway `environment:` includes `CDC_ENABLED=true` and CDC connection vars
- [ ] `deploy/postgres/init.sql` grants `REPLICATION` on user `frf` (or creates `frf_slot`)
- [ ] `admin-ui/e2e/layer2-crdt.spec.ts` exists and passes Layer 1 tests locally
- [ ] `lib/build.gradle.kts` has `tasks.withType<Test> { enabled = false }`
- [ ] Dagger Stage 6 `--no-opt` removed OR `binaryen`/`wasm-opt ≥ 116` installed in stage
- [ ] `cargo check --workspace` still passes after all changes

---

## Open Questions for Plan

1. **wasm-opt approach**: install `binaryen` APT package in Stage 6 Docker build (gives system `wasm-opt`), or upgrade `wasm-bindgen-cli` to 0.2.125 (latest)? The APT route is more predictable; the CLI upgrade is cleaner but may introduce other breaking changes in generated bindings.

2. **Layer 3 CDC test scope**: Phase 4 smoke test (`phase4-smoke.spec.ts`) already has a Layer 3 CDC test block. Should Phase 11 focus on making that existing test pass end-to-end, or add a new targeted Layer 3 CDC test?

3. **Tenant UUID for CDC compose fixture**: CDC requires a `CDC_TENANT_ID` (UUID). Use a well-known fixture UUID (e.g. `00000000-0000-0000-0000-000000000001`) or derive from the test seed?
