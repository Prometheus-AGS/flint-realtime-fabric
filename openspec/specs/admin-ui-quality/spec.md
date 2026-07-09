# admin-ui-quality Specification

## Purpose
TBD - created by archiving change p19-c001-admin-ui-lint-debt. Update Purpose after archive.
## Requirements
### Requirement: The admin-ui lint suite MUST pass clean

`admin-ui` `pnpm lint` MUST exit 0. No source or e2e file may carry a lint error, including
a stale `eslint-disable` directive that references a rule not registered in
`eslint.config.mjs`. Environment-flag guards MUST use explicit `=== "true"` comparison
rather than `!!process.env["X"]` boolean coercion (enforced by the local
`no-boolean-env-coercion` rule).

#### Scenario: lint passes with no errors

- **WHEN** `pnpm lint` runs in `admin-ui`
- **THEN** it exits 0 with zero errors

#### Scenario: env-flag guard uses strict comparison

- **WHEN** an integration-skip flag is read from the environment
- **THEN** it is compared with `=== "true"`, so `SKIP_INTEGRATION=false` does not skip

#### Scenario: no disable directive references an unregistered rule

- **WHEN** a source file would disable `react-hooks/exhaustive-deps`
- **THEN** either the plugin is registered, or (as here) the directive is removed because
  the project does not use that plugin

