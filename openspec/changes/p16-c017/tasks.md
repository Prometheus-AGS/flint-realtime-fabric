# Tasks — p16-c017

- [x] Implement /readyz probing Iggy, Keto, JWKS
- [x] Keep /healthz as a liveness check
- [x] Update compose healthcheck/depends_on to use readiness
- [x] Test: /readyz fails when a dependency is down

## Implementation

### /readyz — real readiness probe (H8)

`routes/health.rs` gained `readyz` (over `AppState`): it probes the three hard
dependencies and returns **503 Service Unavailable** with per-dependency detail when any
is down, **200** when all are reachable:
- **Keto** — HTTP `GET {keto_base_url}/health/ready`
- **JWKS** — HTTP `GET {gateway_jwks_url}`
- **Iggy** — TCP connect to the host:port parsed from `IGGY_CONNECTION_STRING`
  (`iggy://user:pass@host:port`)

All probes use a 3s timeout. Added `reqwest` (workspace dep) to the gateway. Response
body: `{ status, checks: { keto, jwks, iggy: { ok, detail } } }`.

### /healthz — liveness (unchanged, now documented)

Kept the static `{ status: ok, version }` — it is the LIVENESS probe (process is up →
decides RESTART), distinct from readiness (dependencies reachable → decides
add/remove from LB rotation). Doc comments now state the distinction explicitly.

### Compose

- `compose.yml` (production): gateway healthcheck → **`/readyz`**, so anything with
  `depends_on: gateway: condition: service_healthy` waits for real readiness.
- `compose.ci.yml` / `compose.override.yml`: kept **`/healthz`** — those minimal stacks
  exclude flint-gate/keto, so `/readyz` would never pass there; liveness is correct.

## Tests

`health.rs` unit tests (always run):
- `tcp_probe_fails_when_dependency_is_down` — probing a non-listening port → not ok
  (this is what makes /readyz return 503 when a dependency is down).
- `tcp_probe_reports_unparseable_connection_string` — malformed conn → not ok.
- `http_probe_fails_for_unreachable_url` — unreachable URL (TEST-NET-1) → not ok.

Extracted the probe helpers (`probe_tcp`/`probe_http`) so the failure behavior is unit-
testable without constructing a full live `AppState`. A full green-path /readyz needs live
Keto/JWKS/Iggy (integration).

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo test -p frf-gateway --lib` → 11/11 pass (incl. 3 new probe tests)
- `cargo fmt --check` → clean

## Follow-up for docs (G5)

`/readyz` vs `/healthz` semantics + the compose gating choice to be noted in the
deployment runbook (c023).
