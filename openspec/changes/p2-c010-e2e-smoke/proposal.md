# p2-c010 — E2E smoke tests

## Phase
phase-2-generated-sdks

## Depends on
p2-c007 (gateway tonic service), p2-c009 (admin-ui scaffold)

## Directory
`e2e/`

## What this change does

Adds a minimal E2E smoke test suite that validates the gateway and admin UI
together in CI. These tests are the Phase 2 integration gate — they confirm
that the generated SDKs, gateway gRPC surface, and admin UI scaffold are
wired end-to-end correctly.

### Test scope

| Test | Validates |
|------|-----------|
| `healthz.spec.ts` | Gateway `/healthz` returns HTTP 200 |
| `grpc_publish.spec.ts` | gRPC `Spine.Publish` returns without error (unauthenticated → 401) |
| `admin_ui_loads.spec.ts` | Admin UI dev build loads; `<h1>` or nav element visible |

### Tooling

- **Playwright** (no server-side browser extension available in CI)
- Tests run in `chromium` headless mode
- `baseURL` is configurable via `E2E_BASE_URL` env var (default: `http://localhost:3000`)

### Directory structure

```
e2e/
├── package.json
├── playwright.config.ts
└── tests/
    ├── healthz.spec.ts
    ├── grpc_publish.spec.ts
    └── admin_ui_loads.spec.ts
```

### CI integration note

These tests run in the `dagger/` CI pipeline (Phase 0 established this).
The `e2e` pipeline stage must start the gateway binary and admin UI dev server
before running Playwright.

## Exit criteria

- `pnpm install` exits 0 from `e2e/`
- `pnpm test:smoke` (non-gated, gateway NOT running) → `healthz` and `grpc_publish` fail gracefully (connection refused, not crashed)
- `pnpm exec playwright install chromium` exits 0
- Test files have zero TypeScript errors (`pnpm tsc --noEmit`)
- Manual run against a live gateway: all 3 tests pass (documented — not required for CI merge gate)
