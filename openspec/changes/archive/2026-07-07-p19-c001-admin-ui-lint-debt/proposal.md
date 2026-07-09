# p19-c001 — admin-ui lint debt cleanup

## Why

`admin-ui` `pnpm lint` fails with 2 pre-existing errors (flagged as debt in the phase-18
reflection, in files that phase did not touch). The phase-19 goals require
`pnpm lint` to pass clean (G5.2). Both errors reproduce today:

1. `e2e/p7-smoke.spec.ts:21` — `local/no-boolean-env-coercion`: `!!process.env["SKIP_INTEGRATION"]`
   treats the string `"false"` as truthy, so integration tests meant to be skipped run.
   The project's own custom lint rule (`no-boolean-env-coercion`, defined in
   `eslint.config.mjs`) is what catches this — the fix is exactly what the rule prescribes.
2. `src/features/entities/hooks/useEntitySubscription.ts:41` —
   `react-hooks/exhaustive-deps` **"Definition for rule not found."** The file carries an
   inline `// eslint-disable-next-line react-hooks/exhaustive-deps`, but
   `eslint-plugin-react-hooks` is **not** a plugin in this project (`eslint.config.mjs`
   registers only `@typescript-eslint` + the local rule; the plugin is not in
   `package.json`). ESLint errors on a disable directive that references an unregistered
   rule.

## What Changes

1. **`e2e/p7-smoke.spec.ts`** — replace `!!process.env["SKIP_INTEGRATION"]` with
   `process.env["SKIP_INTEGRATION"] === "true"` (the rule's prescribed form; preserves the
   `|| !process.env["GATEWAY_URL"]` half of the guard).
2. **`useEntitySubscription.ts`** — remove the stale
   `// eslint-disable-next-line react-hooks/exhaustive-deps` directive. The project does not
   use `eslint-plugin-react-hooks`, so the directive guards a rule that is never active; the
   effect's dependency array (`[query.channelId, query.consumerId, query.entityType]`) is
   already the correct exhaustive set (module-level `adapter` and the `set*` state setters
   are stable). Removing the directive introduces no real lint gap.

## Non-goals

- Adding `eslint-plugin-react-hooks` (the project deliberately does not use it — YAGNI).
- Touching any other admin-ui file, or the effect's behavior.

## Impact

- Affected: `admin-ui/e2e/p7-smoke.spec.ts`, `admin-ui/src/features/entities/hooks/useEntitySubscription.ts`.
- `pnpm lint` exits 0; `pnpm typecheck` stays green; no runtime behavior change (the
  `p7-smoke` guard now *correctly* skips when `SKIP_INTEGRATION=false` is set — which was
  the latent bug).
