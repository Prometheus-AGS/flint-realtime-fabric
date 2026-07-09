# Tasks — p32-c001-tls-sidecar-secure-context

- [x] 1. Add a Caddy TLS sidecar (+ Caddyfile) to compose.sovereign.yml fronting gateway:8080 on :8443; playwright GATEWAY_URL → https://caddy:8443.
- [x] 2. Chromium flags: --ignore-certificate-errors; remove the p31 unsafe-origin/disable-features flags (real https context needs none).
- [x] 3. Runner brings up caddy; bash-syntax + compose-config clean.
