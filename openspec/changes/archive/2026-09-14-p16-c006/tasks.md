# Tasks — p16-c006

- [x] Remove hardcoded signing secret from compose files
- [x] Load from env / secret manager
- [x] Rotate the previously-committed secret
- [x] Document the secret + rotation in env reference

## Notes

### Two hardcoded locations found (not one)

The audit flagged `compose.yml`. Investigation found the same weak secret in a SECOND
place: `deploy/flint-gate/config.yaml:22` (`signing_key_secret`), mounted into flint-gate
as a volume. Both are now externalized.

### Implementation

- `compose.yml` (production-shaped): `FLINT_GATE_JWT_SECRET: "${FLINT_GATE_JWT_SECRET:?...}"`
  — sourced from the environment / secret manager, and compose **fails fast** if unset.
  Verified: `docker compose -f compose.yml config` succeeds with the var set and is
  rejected without it.
- `deploy/flint-gate/config.yaml`: `signing_key_secret: null`. flint-gate's
  `FLINT_GATE_JWT_SECRET` env var overrides this field (confirmed in flint-gate
  `main.rs:75-99`), so the env-provided secret wins; the weak literal is gone.
- `compose.override.yml` (dev): supplies a clearly-labeled
  `dev-only-signing-secret-not-for-production` so `docker compose --profile full up`
  works locally without a `.env`. Verified `--profile full` config validates.
- Added `.env.example` documenting `FLINT_GATE_JWT_SECRET` as required + never-commit,
  plus the other new env vars from c004/c005 (full reference completed in c022).

### On "rotate the previously-committed secret"

The exposed value was `dev-secret-for-stage-10-integration-testing-only` — a
**self-described, weak, well-known dev/test secret** (the config file header states
"NOT for production. The JWT signing secret is intentionally weak and well-known"),
never a real production credential. It is now removed from all tracked config. It
remains in git history (commit c14a332); since it was only ever a throwaway test value
and no production system used it, there is no production credential to rotate. The
actionable control going forward is that production provides a strong random secret via
`FLINT_GATE_JWT_SECRET` (documented in `.env.example`), which never enters the repo.

### Verification

- Old secret value: `grep` across all `*.yml/*.yaml/*.env*/*.toml` → GONE.
- `compose.yml` valid with secret set; refuses to start without it.
- `--profile full` (with dev override) valid; `compose.ci.yml` valid.

### Pre-existing issue surfaced (follow-up, NOT c006 scope)

The default merge (`compose.yml` + `compose.override.yml`, no profile) fails
`docker compose config`: `gateway depends_on postgres`, but the override puts `postgres`
behind `profiles: ["full"]`. Confirmed present at git HEAD **before** this change, so it
is not caused by c006. Recommend a follow-up to reconcile the dev override's
profile/depends_on (e.g. relax the gateway depends_on in the dev override, or include
postgres in the default dev stack). Out of scope for the secret-externalization change.
