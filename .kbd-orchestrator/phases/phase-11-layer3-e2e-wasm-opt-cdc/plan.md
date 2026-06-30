# Plan — phase-11-layer3-e2e-wasm-opt-cdc

## Change Count: 6

## Ordering Rationale

Changes are ordered in two dependency chains:

**Chain A (CDC + Layer 3 E2E, must sequence):**
1. `p11-c001` — CDC env vars into compose.yml (gateway discovers Postgres CDC config)
2. `p11-c002` — Postgres REPLICATION privilege + slot pre-creation (CDC consumer can connect)
3. `p11-c003` — Dagger Stage 10 GATEWAY_URL fix (Layer 2 tests run inside Dagger)

Chain A must sequence because c003 needs c001+c002 for a meaningful Layer 3 E2E run.
c001 and c002 are independent of each other but both must land before c003.

**Chain B (independent fixes, any order after Chain A or in parallel):**
4. `p11-c004` — CRDT Layer 2 Playwright spec (new test file, no blocking deps)
5. `p11-c005` — Kotlin JNI test guard (Gradle build.gradle.kts change only)
6. `p11-c006` — wasm-opt binaryen upgrade + remove --no-opt (Dagger Stage 6 change)

c004/c005/c006 are independent of Chain A and each other — execute after Chain A
completes, or opportunistically interleaved if the executor supports parallelism.

## Ordered Change List

| Order | Change ID | Description | Depends On |
|-------|-----------|-------------|------------|
| 1 | p11-c001-cdc-compose-wiring | Add CDC env vars to compose.yml gateway | — |
| 2 | p11-c002-postgres-replication-setup | ALTER USER frf REPLICATION + slot creation | — |
| 3 | p11-c003-layer3-e2e-run | Dagger Stage 10: inject GATEWAY_URL + SKIP_INTEGRATION | c001, c002 |
| 4 | p11-c004-crdt-layer2-test | layer2-crdt.spec.ts E2E spec | — |
| 5 | p11-c005-kotlin-jni-test-guard | Gradle tasks.withType<Test> { enabled = false } | — |
| 6 | p11-c006-wasm-opt-upgrade | binaryen 116 install, remove --no-opt | — |

## Recommended Execution Approach

Execute c001 and c002 in parallel (both touch different files with no overlap),
then c003. Then execute c004, c005, c006 in parallel.

**Minimum sequential depth:** 2 serial steps (c001+c002 parallel → c003 → c004/c005/c006 parallel)

## Files Touched Summary

| Change | Files Modified/Created |
|--------|------------------------|
| c001 | `compose.yml` |
| c002 | `deploy/postgres/init.sql` |
| c003 | `dagger/codegen.ts` |
| c004 | `admin-ui/e2e/layer2-crdt.spec.ts` (NEW) |
| c005 | `sdks/kotlin/lib/build.gradle.kts` |
| c006 | `dagger/codegen.ts` |

Note: c003 and c006 both touch `dagger/codegen.ts` — execute sequentially
(c003 first, c006 second) or apply both edits in c003 if the executor merges.

## Exit Criteria (Phase Level)

All 6 changes complete AND:
- `docker compose config` validates after c001
- `docker compose exec postgres psql -U frf -c "\du"` shows Replication attr after c002
- Dagger Stage 10 includes GATEWAY_URL in env block after c003
- `admin-ui/e2e/layer2-crdt.spec.ts` listed by playwright after c004
- `cd sdks/kotlin && ./gradlew test` exits 0 (tests skipped) after c005
- `wasm-opt --version` outputs version_116 in Stage 6 after c006
