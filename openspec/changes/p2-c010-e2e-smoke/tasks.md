# Tasks — p2-c010 e2e-smoke

- [ ] **T1** Create `e2e/package.json`
  - Name: `@prometheusags/frf-e2e`
  - Type: `"module"`
  - DevDependencies: `@playwright/test@^1.53.0`, `typescript@^5.8.3`
  - Scripts: `test:smoke: playwright test`, `install:browsers: playwright install chromium`
  - Verification: valid JSON; `pnpm install` exits 0

- [ ] **T2** Create `e2e/playwright.config.ts`
  - `testDir: ./tests`
  - `use.baseURL`: `process.env.E2E_BASE_URL ?? 'http://localhost:3000'`
  - `projects`: chromium only, headless
  - `reporter`: `['list']` for CI
  - `timeout`: 10_000ms per test (smoke tests must be fast)
  - Verification: `pnpm exec playwright list-files` exits 0

- [ ] **T3** Create `e2e/tests/healthz.spec.ts`
  - `test('gateway /healthz returns 200')`:
    ```ts
    const response = await request.get('/healthz');
    expect(response.status()).toBe(200);
    ```
  - Uses `@playwright/test` `request` fixture (no browser needed)
  - Verification: file has zero TypeScript errors

- [ ] **T4** Create `e2e/tests/grpc_publish.spec.ts`
  - `test('grpc Spine.Publish without token returns unauthenticated')`:
    - Uses `request.post('/flint.v1.SpineService/Publish', { ... })` with Connect-protocol headers
    - Assert status is 401 or 16 (gRPC Unauthenticated) — not 5xx
  - Verification: file has zero TypeScript errors

- [ ] **T5** Create `e2e/tests/admin_ui_loads.spec.ts`
  - `test('admin UI loads with visible navigation')`:
    ```ts
    await page.goto('/');
    await expect(page.locator('nav, [role=navigation]')).toBeVisible();
    ```
  - `baseURL` must point to admin UI port (`ADMIN_UI_URL` env or `http://localhost:5173`)
  - Verification: file has zero TypeScript errors

- [ ] **T6** TypeScript check
  - Create `e2e/tsconfig.json` with `strict: true` and `moduleResolution: NodeNext`
  - `pnpm tsc --noEmit` from `e2e/`
  - Verification: exits 0

- [ ] **T7** Install Playwright browser
  - `pnpm exec playwright install chromium`
  - Verification: chromium binary downloaded; `playwright --version` exits 0

- [ ] **T8** Run smoke tests (connection-refused mode)
  - `pnpm test:smoke` with no gateway running
  - Verification: tests FAIL due to connection refused (not due to test syntax errors or import errors); exit code is non-zero but no unhandled exceptions
