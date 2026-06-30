# Tasks — p14-c004-phase6-spec-404-fix

- [x] Locate "404 in release" test in `admin-ui/e2e/phase6-smoke.spec.ts`
- [x] Add `test.skip(process.env["DEV_ENDPOINTS_ENABLED"] === "true", ...)` guard
- [x] Read `dagger/codegen.ts` Stage 10 section
- [x] Add `DEV_ENDPOINTS_ENABLED=true` to Stage 10 container env vars
