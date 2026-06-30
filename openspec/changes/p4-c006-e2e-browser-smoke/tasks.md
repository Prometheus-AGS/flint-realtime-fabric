# Tasks — p4-c006 e2e-browser-smoke

- [ ] **T1** Create `docker-compose.test.yml` for Phase 4 test stack
  - File: `docker-compose.test.yml` (workspace root)
  - Services: `postgres` (image: postgres:17-alpine, wal_level=logical),
    `iggy` (image: iggyrs/iggy:latest)
  - Environment variables for gateway: CDC_ENABLED=true, CDC_REPLICATION_URL,
    IGGY_CONNECTION_STRING, stub KETO/OATHKEEPER vars pointing to localhost noops
  - Verification: `docker compose -f docker-compose.test.yml config` exits 0

- [ ] **T2** Create `tests/browser/` directory and Playwright config
  - Files: `tests/browser/playwright.config.ts`, `tests/browser/package.json`
  - `playwright.config.ts`: baseURL = `http://localhost:3000`, single chromium project,
    webServer config pointing to `pnpm --prefix admin-ui dev`
  - Verification: directory structure exists; `npx playwright --version` reports a version

- [ ] **T3** Implement Phase 4 exit-criterion smoke test
  - File: `tests/browser/phase4.smoke.spec.ts`
  - Test `phase4_exit_criterion`:
    1. Navigate to `/demo/signaling`
    2. Assert page `h1` visible
    3. Assert CRDT demo button exists
    4. Click CRDT demo button, assert `pre` element with content appears within 3s
    5. (CDC path, requires Docker stack): check `[data-testid="entity-event-count"]`
       increments after a `psql` INSERT — mark as `test.skip` when Docker not present
  - Test `wasm_loads_without_errors`:
    1. Intercept console errors before navigation
    2. Navigate to `/demo/signaling`
    3. Assert zero console errors containing "wasm" or "WebAssembly"
  - Verification: `npx playwright test tests/browser/phase4.smoke.spec.ts` exits 0
    (CRDT test only; CDC test skipped without Docker)

- [ ] **T4** Add `pnpm test:e2e:phase4` script to admin-ui
  - File: `admin-ui/package.json`
  - Add script: `"test:e2e:phase4": "playwright test --config=../tests/browser/playwright.config.ts"`
  - Verification: script present in package.json

- [ ] **T5** Document Phase 4 test run procedure
  - File: `docs/testing/phase4-e2e.md` (new)
  - Instructions: start Docker stack, build wasm SDK, run gateway, run Playwright
  - Include env var checklist
  - Verification: file exists and is under 200 lines
