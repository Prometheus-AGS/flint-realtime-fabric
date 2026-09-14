# Tasks — p4-c001 cdc-gateway-wiring

- [ ] **T1** Add `frf-postgres-cdc` dependency to `frf-gateway/Cargo.toml`
  - Add `frf-postgres-cdc = { path = "../frf-postgres-cdc" }` under `[dependencies]`
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T2** Extend `GatewayConfig` with CDC fields
  - File: `crates/frf-gateway/src/config.rs`
  - Add to struct: `cdc_replication_url: String`, `cdc_slot_name: String`,
    `cdc_publication_name: String`, `cdc_tenant_id: uuid::Uuid`,
    `cdc_channel_path: String`, `cdc_enabled: bool`
  - Source from env vars: `CDC_REPLICATION_URL`, `CDC_SLOT_NAME`,
    `CDC_PUBLICATION_NAME`, `CDC_TENANT_ID`, `CDC_CHANNEL_PATH`, `CDC_ENABLED`
  - `cdc_enabled` defaults to `false` if env var absent (opt-in)
  - Verification: `cargo check -p frf-gateway` exits 0; no unwrap()

- [ ] **T3** Construct and spawn `PostgresCdcConsumer` in `main.rs`
  - File: `crates/frf-gateway/src/main.rs`
  - Create a `tokio::sync::watch::channel(false)` for shutdown signaling
  - If `config.cdc_enabled`: construct `CdcConfig` from gateway config, build
    `PostgresCdcConsumer::new(cdc_config, Arc::clone(&broker))`, spawn
    `tokio::spawn(consumer.run_until_shutdown(shutdown_rx))`
  - On SIGTERM / ctrl-c: send `true` to the watch channel before awaiting the task
  - Verification: `cargo build -p frf-gateway` exits 0; no unwrap()/expect() in
    library paths

- [ ] **T4** Add CDC integration smoke test
  - File: `crates/frf-postgres-cdc/tests/consumer_smoke.rs`
  - Test `cdc_config_builds_from_env`: set env vars, build `CdcConfig`, assert
    fields round-trip correctly (no Postgres connection required)
  - Verification: `cargo test -p frf-postgres-cdc` exits 0
