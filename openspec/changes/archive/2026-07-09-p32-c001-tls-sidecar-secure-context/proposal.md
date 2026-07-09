# p32-c001-tls-sidecar-secure-context

## Why

Phase-31: the in-network decode browser navigates to the insecure origin `http://gateway:8080`, so
`navigator.mediaDevices` is undefined and `getUserMedia` throws before any offer; the
`--unsafely-treat-insecure-origin-as-secure-origin` flags did not register `mediaDevices` in the
headless Playwright container. A browser needs a genuine secure context (HTTPS) for `getUserMedia` —
unsafe flags are unreliable. Front the gateway with a TLS sidecar so the browser reaches `https://`.

## What Changes

- **Caddy TLS sidecar** in `compose.sovereign.yml`: `reverse_proxy gateway:8080` (Caddy proxies HTTP
  **and** the WebSocket upgrade), auto internal self-signed cert, listening on `:8443`, on the compose
  network, `depends_on: gateway healthy`.
- **Harness:** the Playwright service's `GATEWAY_URL` → `https://caddy:8443` (the spec derives
  `WS_URL = replace(/^http/, "ws")` → `wss://caddy:8443` + `goto`s the https origin automatically).
- **Chromium flags:** launch with `--ignore-certificate-errors` (trust the self-signed) and **remove**
  the p31 `--unsafely-treat-insecure-origin-as-secure-origin` / `--disable-features` /
  `--disable-site-isolation-trials` — a real `https://` context needs none.
- **Runner:** `scripts/run-media-decode.sh` brings up `caddy` too.

## Impact

- `compose.sovereign.yml` (caddy service + Caddyfile), `admin-ui/e2e/media-decode.spec.ts` (flags),
  `scripts/run-media-decode.sh` (bring up caddy). No `frf-*` engine change (a TLS proxy is not a
  gateway-transport change).
