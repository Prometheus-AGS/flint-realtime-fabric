# Tasks — p19-c001-admin-ui-lint-debt

- [x] 1. Fix `e2e/p7-smoke.spec.ts:21` — `!!process.env["SKIP_INTEGRATION"]` → `process.env["SKIP_INTEGRATION"] === "true"`
- [x] 2. Remove the stale `// eslint-disable-next-line react-hooks/exhaustive-deps` in `useEntitySubscription.ts` (rule not registered; deps already exhaustive)
- [x] 3. Verify `pnpm lint` exits 0 and `pnpm typecheck` stays green
