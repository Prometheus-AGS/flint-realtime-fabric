# Assessment — phase-32-sovereign-sfu-decode-https-secure-context-and-flip

> Date: 2026-07-09. Gap report against G1 (give the in-network browser a genuine secure context) +
> G2 (decode + flip) + G3 (carried). Grounded in the harness URL flow + the gateway's transport.

## Method

Read `admin-ui/e2e/media-decode.spec.ts` + `decode-probe.ts` (how the browser reaches the gateway
over HTTP + WS), `compose.yml` (gateway transport), and the p31 secure-context evidence. No code
changed (assess only).

## Current state (what exists)

- **Both origins derive from `GATEWAY_URL`.** The spec sets `WS_URL = GATEWAY_URL.replace(/^http/,
  "ws")` and the page `goto`s `${GATEWAY_URL}/`, the WS connects to `${wsUrl}/ws/v1/signal`. So a
  single `GATEWAY_URL=https://gateway:8443` yields **`https://` for the page AND `wss://` for the
  signaling** automatically — no per-URL edits.
- **The gateway serves plain HTTP on `8080`** (`BIND_ADDR: 0.0.0.0:8080`, `compose.yml`); it does
  **not** terminate TLS. Gateway-terminated TLS would need a Rust change (rustls/axum-server + cert
  plumbing) — heavier, and an engine change.
- **p31 blocker:** the in-network browser at insecure `http://gateway:8080` has
  `navigator.mediaDevices === undefined`; unsafe-origin flags did not register it in the headless
  Playwright container.
- **Media path proven/in-place** (candidate IP, shared socket, mDNS skip, STUN, in-network topology);
  the prebuilt-image runner + in-network Playwright + DECODE_ONLY + coturn wiring all exist.

## Gap analysis

### G1 — genuine secure context for the in-network browser  ·  **GAP: CONFIRMED (the phase core)**

- **Defect:** `getUserMedia`/`mediaDevices` require a secure context; `http://gateway:8080` is
  insecure and the unsafe-origin flags don't take (p31).
- **Two candidate approaches, assessed:**

  | Approach | How | Verdict |
  |---|---|---|
  | **TLS sidecar (Caddy) fronting the gateway** | A `caddy` service on the compose network with an auto self-signed internal cert, `reverse_proxy gateway:8080` (Caddy proxies HTTP **and** the WebSocket upgrade transparently). Harness `GATEWAY_URL=https://caddy:8443` → page is `https://` (secure context) + WS is `wss://`. Browser launched with `--ignore-certificate-errors` to trust the self-signed. **No gateway engine change.** | **RECOMMENDED** |
  | **`*.localhost` origin alias** | Reach the gateway as `gateway.localhost` (Chromium treats `*.localhost` as trustworthy → secure). | **Risky:** many resolvers special-case `*.localhost` → `127.0.0.1`, which *inside* the browser container is the browser itself, not the gateway — the WS/page would not reach the SFU. Rejected as primary. |
  | **Gateway terminates TLS** | rustls/axum-server + cert in `frf-gateway`. | Works but is an **engine change** (ADR) for a proof-harness need. Deferred unless the sidecar is insufficient. |

  **Recommendation: Caddy TLS sidecar.** It gives a real `https://`/`wss://` secure context, proxies
  both HTTP and the WS upgrade the signaling needs, requires **no gateway engine change**, and drops
  the fragile p31 unsafe-origin flags entirely.

- **Scope (sidecar):** `compose.sovereign.yml` (a `caddy` service + a tiny Caddyfile, on the compose
  net, `depends_on: gateway healthy`); `scripts/run-media-decode.sh` (bring up `caddy`; set the
  in-network `GATEWAY_URL=https://caddy:8443`; the health-check + JWKS/Keto host-side steps unchanged);
  `admin-ui/e2e/media-decode.spec.ts` (launch with `--ignore-certificate-errors`; **remove** the p31
  `--unsafely-treat-insecure-origin-as-secure-origin` / `--disable-features` flags — a real secure
  context needs none). No `frf-*` engine change. File-size ≤500.

- **Risk:** medium. Caddy's self-signed cert must be trusted (`--ignore-certificate-errors` covers
  it); the WS upgrade must pass through (Caddy does this by default). The gateway's own health/JWKS
  paths are host-side and unaffected. If TLS fronting still doesn't yield `mediaDevices` (unlikely —
  `https://` is unambiguously a secure context), that is the **environment-pivot signal** written into
  the phase goals.

### G2 — decoded frame + flip  ·  **GAP: OPEN (depends on G1)**

Unchanged discipline: re-run; observe `ice=connected` + `framesDecoded > 0`; flip only on a genuine
frame, else re-affirm gated. This is the run where the whole media path is finally exercised
end-to-end — the sender can acquire a track once the secure context lands.

### G3 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated. No new finding.

## Open question for plan/analyze

**None blocking.** The Caddy TLS sidecar is the clear, no-engine-change fix; `*.localhost` is too
risky in-container and gateway-terminated TLS is an unnecessary engine change for a harness need. The
plan should confirm the sidecar and keep the **environment-pivot decision point** (from the phase
goals) live: if HTTPS still doesn't produce a decoded frame, move the proof to CI / another host
rather than peel a 10th layer.

## Recommended change ordering (for plan)

1. **c001 — TLS sidecar + secure-context harness** (Caddy fronting the gateway; `GATEWAY_URL=https`;
   `--ignore-certificate-errors`; drop the unsafe flags). Unblocks `getUserMedia`.
2. **c002 — decode re-run + conditional flip** (the whole media path exercised; PHASE-32-DECODE-RESULT;
   flip or re-affirm — and, if it's another harness layer, invoke the environment-pivot decision).

## Exit posture

G1 is a harness/compose fix (Caddy sidecar), not engine code, and it addresses the exact p31 blocker
with a real secure context. G2 is the honest gate — and the 9th decode attempt, where the media path
finally runs end-to-end. The environment-pivot decision point stays explicit. Nothing here re-opens an
ADR (a TLS *proxy* is not a gateway-transport change).
