# Tasks — p14-c005-enable-federation-stage

- [x] Locate Phase 6 Layer 3 describe block in `admin-ui/e2e/phase6-smoke.spec.ts`
- [x] Add `ENABLE_FEDERATION` constant at top of file
- [x] Add `test.skip(!ENABLE_FEDERATION, ...)` guard to Layer 3 describe block
- [x] Verify Stage 10 in `dagger/codegen.ts` does NOT set `ENABLE_FEDERATION_STAGE`
