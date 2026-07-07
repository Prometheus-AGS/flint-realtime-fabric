# Tasks — p16-c022

- [x] Enumerate all gateway env vars
- [x] Document purpose/required/secret/dev-only for each
- [x] Flag DEV_NO_AUTH as a dev-only bypass
- [x] Write .env.example

## Summary (H14 — env vars documented)

### Enumeration (task 1)

Grepped every `std::env::var("…")` / clap `env = "…"` across the gateway and its
media adapters — **37 variables**. This is the authoritative, exhaustive list (no more
"~30 undocumented vars").

### Reference doc (tasks 2, 3) — `docs/ENVIRONMENT.md`

Every var in a grouped table with columns Req / Secret / Dev-only / Default / Purpose:
core/networking, identity & authz, security middleware, media/SFU, CDC, federation,
registry, observability. Explicitly marks:
- **Required to boot**: `IGGY_CONNECTION_STRING`, `KETO_BASE_URL`, `GATEWAY_JWKS_URL`,
  `JWT_AUDIENCE`.
- **Secrets** (never commit): `IGGY_CONNECTION_STRING`, `FLINT_GATE_JWT_SECRET`,
  `LIVEKIT_API_KEY/SECRET`, `CDC_REPLICATION_URL`, `MATRIX_ACCESS_TOKEN`.
- **Mode-conditional required** (⚠️): `LIVEKIT_*` when `SFU_MODE=hosted`, `CDC_*` when
  `CDC_ENABLED`, `FEDERATION_TENANT_ID` when federation enabled — all fail-fast at boot
  (p16-c020).
- **DEV_NO_AUTH** flagged as a **DEV-ONLY BYPASS**, "never set in production" (task 3).

### .env.example (task 4)

Rewrote from the partial c006/c009/c010 seed into the complete template: the 4
required-to-boot vars uncommented with sample values, everything else commented with
defaults, grouped to mirror the reference, secrets and dev-only bypass clearly labeled,
and a pointer to `docs/ENVIRONMENT.md`.

## Verification

Cross-checked all 37 enumerated vars against both files → **none missing** from
`.env.example` or `docs/ENVIRONMENT.md`.
