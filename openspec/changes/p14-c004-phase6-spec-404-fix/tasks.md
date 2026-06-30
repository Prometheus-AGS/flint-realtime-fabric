# Tasks — p14-c004-phase6-spec-404-fix

- [ ] Locate "404 in release" test in `admin-ui/e2e/phase6-smoke.spec.ts`
- [ ] Add `test.skip(process.env["DEV_ENDPOINTS_ENABLED"] === "true", ...)` guard
- [ ] Read `dagger/codegen.ts` Stage 10 section
- [ ] Add `DEV_ENDPOINTS_ENABLED=true` to Stage 10 container env vars
