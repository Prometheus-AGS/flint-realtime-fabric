# Tasks — p30-c001-browser-in-docker-harness

- [x] 1. Add a Playwright/Chromium service (v1.52.0) to compose.sovereign.yml on the compose network with admin-ui mounted.
- [x] 2. Secure-context Chromium launch: --unsafely-treat-insecure-origin-as-secure-origin for the in-network gateway origin (sender + receiver contexts).
- [x] 3. Runner runs the spec inside the Playwright container (GATEWAY_URL=http://gateway:8080, STUN_URL=stun:coturn:3478); host JWT/JWKS/Keto path intact.
