# Development Guide

This guide covers local development workflows for `flint-realtime-fabric`.
For the full build plan see [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md).
For agent rules see [../CLAUDE.md](../CLAUDE.md) and [PROMETHEUS-BASE-RULES.md](PROMETHEUS-BASE-RULES.md).

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | ≥1.82 (see `rust-toolchain.toml`) | Core workspace |
| `cargo` | (bundled with Rust) | Build + test |
| `wasm-pack` | ≥0.13 | WASM build (Stage 6) |
| `wasm-opt` | from `binaryen` | WASM optimisation |
| Node.js | ≥24 (see `.node-version`) | TypeScript SDK + admin-UI |
| pnpm | ≥9 | JS package manager |
| Docker + Compose v2 | latest | Full-stack tests |
| Dagger | latest | CI pipeline (optional for local dev) |
| `protoc` | ≥3.21 | Proto codegen |

---

## Local development

### Quick start (Rust only)

```bash
cargo check --workspace       # fast compilation check
cargo test --workspace        # run all tests
make clippy                   # CI-equivalent clippy
make fmt                      # format all code
```

### Admin UI

```bash
cd admin-ui
pnpm install
pnpm dev           # dev server at http://localhost:5173
pnpm typecheck     # TypeScript check only
pnpm lint          # ESLint (admin-ui only)
pnpm test:e2e      # Playwright E2E — Layer 1 only, no gateway needed
```

---

## Full compose stack

Start the full stack (gateway + iggy + keto + oathkeeper + surrealdb + postgres):

```bash
make compose-up
# or: docker compose up -d
```

Wait for the gateway to be healthy:

```bash
curl -sf http://localhost:28080/healthz && echo "OK"
```

Stop and clean up:

```bash
make compose-down
# or: docker compose down
```

### Port map

| Service | External port | Internal port | Notes |
|---------|--------------|---------------|-------|
| gateway HTTP | 28080 | 8080 | `/healthz`, `/v1/publish`, `/ws/v1/subscribe` |
| gateway gRPC | 29090 | 9090 | tonic gRPC + Connect-ES |
| iggy | 8090 | 8090 | Apache Iggy broker |
| keto read | 4466 | 4466 | Ory Keto relation API |
| keto write | 4467 | 4467 | Ory Keto write API |
| oathkeeper proxy | 14455 | 4455 | Oathkeeper proxy |
| oathkeeper API | 14456 | 4456 | JWKS / health |
| SurrealDB | 8001 | 8000 | REST + WS |
| PostgreSQL | 15432 | 5432 | CDC + storage |

### CDC slot smoke test

After `compose-up`, verify the CDC replication slot is active:

```bash
make cdc-smoke
# or: bash scripts/smoke-cdc.sh
```

Expected output: `OK: CDC slot 'frf_slot' is active after Ns`

If the slot is not active within 30 seconds:
- Check gateway logs: `docker compose logs gateway | tail -50`
- Verify CDC env vars in `compose.yml` (`CDC_ENABLED`, `CDC_REPLICATION_URL`, `CDC_SLOT_NAME`)
- Increase timeout: `CDC_SMOKE_TIMEOUT=60 bash scripts/smoke-cdc.sh`

---

## WASM binary

### Build WASM locally

```bash
cd crates/frf-wasm
./build_wasm.sh
```

Output: `sdks/ts/frf-wasm/frf_wasm_bg.wasm`

Requires `wasm-pack` and `wasm-opt` (binaryen ≥116):

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install binaryen (macOS)
brew install binaryen

# Install binaryen (Debian/Ubuntu)
apt-get install binaryen
```

### Establish or update the WASM size baseline

The WASM size gate in Dagger Stage 6 compares the binary size against
`.wasm-size-baseline` (a single integer: byte count). Run this after any
intentional WASM size increase:

```bash
make baseline-wasm
```

This runs the full Dagger pipeline through Stage 6, measures the binary,
writes `.wasm-size-baseline`, and creates a git commit. Requires Docker + Dagger.

The gate uses a 150% threshold: a binary more than 50% larger than the baseline
causes Stage 6 to fail with `FAIL: WASM size N > 150% of baseline B`.

---

## Layer 3 E2E (Stage 10)

Stage 10 requires a Docker host with DinD (`/var/run/docker.sock` mounted into
the Dagger runner, or a `--privileged` CI runner):

```bash
make layer3-e2e
# or: ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```

The pipeline:

1. Builds the gateway image with `CARGO_FEATURES=dev-endpoints` (enables `/dev/*` routes).
2. Starts `docker compose up -d`.
3. Waits for iggy to be healthy (compose `depends_on: service_healthy`).
4. Polls `http://localhost:8080/healthz` for up to 60s.
5. Runs `pnpm exec playwright test e2e/` with:
   - `SKIP_INTEGRATION=false`
   - `WASM_AVAILABLE=1`
   - `GATEWAY_URL=http://localhost:8080`
6. Tears down `docker compose down`.

### Dev endpoint note

The federation smoke tests (`phase6-smoke.spec.ts`) POST to
`/dev/inject-federation-event`. This endpoint is only compiled when the gateway
is built with `--features dev-endpoints`. The `compose.yml` build args enable
this automatically; production images built without `CARGO_FEATURES` do not
expose the endpoint.

### CDC timing

If CDC smoke tests time out, the slot may not activate before the test runs.
Increase the poll timeout:

```bash
CDC_SMOKE_TIMEOUT=60 bash scripts/smoke-cdc.sh
```

---

## Architecture Decision Records

ADRs live in `docs/decisions/`. Add new ADRs as `adr-NNN-<slug>.md`.
See [decisions/adr-001-crdt-engine.md](decisions/adr-001-crdt-engine.md) for
the format.
