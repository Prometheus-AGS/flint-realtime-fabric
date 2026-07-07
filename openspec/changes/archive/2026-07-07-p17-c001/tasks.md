# Tasks — p17-c001

- [x] Add compose.override.yml to .gitignore; ship compose.override.example.yml with placeholders
- [x] Remove the committed dev-secret literal; use ${FLINT_GATE_JWT_SECRET:-dev...} sourced from .env
- [x] Add a top-of-file guard comment: NEVER use as a deploy base
- [x] Verify a bare `docker compose config` no longer surfaces a committed secret
