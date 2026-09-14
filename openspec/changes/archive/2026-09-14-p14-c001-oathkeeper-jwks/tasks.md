# Tasks — p14-c001-flint-gate-compose

- [x] Replace `oathkeeper` service in `compose.yml` with `flint-gate` service
- [x] Create `deploy/flint-gate/config.yaml` — dev passthrough config with JWT minting
- [x] Rename `OATHKEEPER_JWKS_URL` → `GATEWAY_JWKS_URL` in config.rs, main.rs, compose.yml
- [x] Point `GATEWAY_JWKS_URL` at flint-gate admin signing-keys endpoint (`http://flint-gate:4457/signing-keys`)
- [x] Delete `deploy/oathkeeper/` directory
