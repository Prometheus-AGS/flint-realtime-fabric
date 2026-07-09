# Tasks — p19-c002-atproto-outbound-gateway-wiring

- [x] 1. config.rs: add PDS-writer fields (`atproto_pds_url`, `atproto_pds_identifier`, `atproto_pds_app_password`, `atproto_write_collection`) + `from_env` loading + `test_default` defaults (all None/empty)
- [x] 2. config.rs::validate: all-or-none guard for the 3 PDS-writer vars; unit tests (all-set passes, partial fails naming the var)
- [x] 3. main.rs: in `build_federation_bridges`, build `AtProtoBridge::with_writer(PdsConfig, collection)` when the 3 fields are present; update the log; keep inbound-only path otherwise
- [x] 4. ENVIRONMENT.md: document the 4 new env vars; mark app-password as a secret
- [x] 5. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green (workspace lib/bins)
