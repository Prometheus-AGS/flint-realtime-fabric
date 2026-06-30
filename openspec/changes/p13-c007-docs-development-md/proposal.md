# p13-c007 — Create `docs/DEVELOPMENT.md`

## Summary

Create a quick-start development guide covering the topics most likely to
block a developer new to this workspace: local compose stack, WASM
baselining, CDC slot verification, and Layer 3 E2E requirements.

## File to create

- `docs/DEVELOPMENT.md`

## Specification

```markdown
# Development Guide

This guide covers local development workflows for `flint-realtime-fabric`.
For the full build plan see [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md).
For agent rules see [../CLAUDE.md](../CLAUDE.md) and
[PROMETHEUS-BASE-RULES.md](PROMETHEUS-BASE-RULES.md).

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | ≥1.82 (see `rust-toolchain.toml`) | Core workspace |
| `cargo` | (bundled with Rust) | Build + test |
| `wasm-pack` | ≥0.13 | WASM build (Stage 6) |
| `wasm-opt` | (from `binaryen`) | WASM optimisation |
| Node.js | ≥24 (see `.node-version`) | TypeScript SDK + admin-UI |
| pnpm | ≥9 | JS package manager |
| Docker + Compose | v2+ | Full-stack tests |
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
pnpm typecheck     # type check only
pnpm test:e2e      # Playwright E2E (Layer 1 only — no gateway needed)
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

Stop:

```bash
make compose-down
# or: docker compose down
```

### CDC slot smoke test

After `compose-up`, verify the CDC replication slot is active:

```bash
make cdc-smoke
# or: bash scripts/smoke-cdc.sh
```

Expected output: `OK: CDC slot 'frf_slot' is active after Ns`

---

## WASM binary

### Build WASM

```bash
cd crates/frf-wasm
./build_wasm.sh
# or via Dagger Stage 6: dagger run ts-node dagger/codegen.ts
```

Output: `sdks/ts/frf-wasm/frf_wasm_bg.wasm`

### Establish or update the WASM size baseline

The WASM size gate in Dagger Stage 6 compares the binary size against
`.wasm-size-baseline` (a single integer: byte count). Run this after any
intentional WASM size change:

```bash
make baseline-wasm
```

This runs the full pipeline, measures the binary, writes `.wasm-size-baseline`,
and creates a git commit. Requires Docker + Dagger.

---

## Layer 3 E2E (Stage 10)

Stage 10 requires a Docker host with DinD (`/var/run/docker.sock` mounted into
the Dagger runner, or a `--privileged` CI runner):

```bash
make layer3-e2e
# or: ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```

The pipeline:
1. Builds the gateway image with the `dev-endpoints` Cargo feature enabled.
2. Starts `docker compose up -d`.
3. Polls `http://localhost:8080/healthz` for up to 60s.
4. Runs `pnpm exec playwright test e2e/` with `SKIP_INTEGRATION=false`,
   `WASM_AVAILABLE=1`, and `GATEWAY_URL=http://localhost:8080`.
5. Tears down `docker compose down`.

### CDC timing note

If the CDC smoke tests fail with slot not active, increase
`CDC_SMOKE_TIMEOUT` (default: 30s):

```bash
CDC_SMOKE_TIMEOUT=60 bash scripts/smoke-cdc.sh
```

---

## Architecture Decision Records

ADRs live in `docs/decisions/`. Add new ADRs as `adr-NNN-<slug>.md`.
See [adr-001-crdt-engine.md](decisions/adr-001-crdt-engine.md) for the
template.
```

## Acceptance criteria

1. `docs/DEVELOPMENT.md` renders correctly in GitHub Markdown.
2. All `make` commands referenced exist in the `Makefile` added by c006.
3. Port numbers and service names match `compose.yml`.
4. No hardcoded secrets or credentials.
