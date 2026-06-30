# Tasks — p14-c005-enable-federation-stage

- [ ] Locate Phase 6 Layer 3 describe block in `admin-ui/e2e/phase6-smoke.spec.ts`
- [ ] Add `ENABLE_FEDERATION_STAGE` env var constant at top of file (or describe block)
- [ ] Add `test.skip(!ENABLE_FEDERATION, ...)` guard to Layer 3 describe block
- [ ] Verify Stage 10 in `dagger/codegen.ts` does NOT set `ENABLE_FEDERATION_STAGE`
