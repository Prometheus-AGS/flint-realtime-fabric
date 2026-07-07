# Tasks — p16-c020

- [x] Add semantic validation for media/SFU config
- [x] Fail fast on empty LiveKit creds in hosted mode
- [x] Validate other mode-specific required fields
- [x] Test: invalid config aborts boot with a clear error

## Implementation (#36 — fail fast on invalid config)

### The defect

`build_media_signaler` silently degraded on empty LiveKit creds — a hosted-SFU
deployment with missing `LIVEKIT_*` env booted "successfully" but signaling did
nothing. Config validation checked env-var PRESENCE only, not semantic validity.

### GatewayConfig::validate()

New method, called in `main` immediately after `from_env` (before building
anything), that fails fast with a clear message on the first semantic violation:
- **Hosted SFU** (`SFU_MODE=hosted`) → `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`,
  `LIVEKIT_SERVER_URL` must be set to non-empty values (the LiveKit creds live in
  env, not the config struct, so validate reads them there).
- **CDC** (`CDC_ENABLED=true`) → `CDC_REPLICATION_URL`, `CDC_SLOT_NAME`,
  `CDC_PUBLICATION_NAME` must be present (previously only caught later, inside
  `spawn_cdc_consumer`).
- **Federation** (`FEDERATION_ENABLED=true`) → `FEDERATION_TENANT_ID` must be set
  (reinforces c009's guard at the config layer).

Each `anyhow::ensure!` names the offending env var, so a misconfiguration aborts
boot with an actionable message instead of a silent broken state.

## Tests (`config.rs`)

- `valid_default_config_passes_validation` — sovereign + CDC off + fed off → ok.
- `cdc_enabled_without_replication_url_fails_fast` — error names `CDC_REPLICATION_URL`.
- `cdc_enabled_with_all_fields_passes`.
- `federation_enabled_without_tenant_fails_fast` — error names `FEDERATION_TENANT_ID`.
- `hosted_sfu_without_livekit_creds_fails_fast` — error names `LIVEKIT_` (guarded to
  skip if the env happens to provide creds, avoiding a flaky false-negative).

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo test -p frf-gateway --lib` → 17/17 pass (incl. 5 new validate tests)
- `cargo fmt --check` → clean
