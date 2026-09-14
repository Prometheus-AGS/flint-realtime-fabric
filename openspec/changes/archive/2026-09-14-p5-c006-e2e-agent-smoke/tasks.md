# Tasks — p5-c006-e2e-agent-smoke

- [ ] Create `admin-ui/e2e/phase5-smoke.spec.ts`
- [ ] Layer 1 tests (UI, no gateway): panel renders, empty state visible, no crash on WS close
- [ ] Layer 2 tests (WS, gated on GATEWAY_URL): `/ws/v1/agents` returns 101 upgrade
- [ ] Layer 3 tests (E2E, gated on GATEWAY_URL): agent event appears in panel after RunAgent call
- [ ] Use `test.skip(SKIP_INTEGRATION, ...)` pattern for layers 2 and 3
- [ ] All selectors use `data-testid` attributes
- [ ] `pnpm test:e2e` passes (layer 1 only in CI)
