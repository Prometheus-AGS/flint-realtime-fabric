---
id: quickstart
title: Quickstart
sidebar_label: Quickstart
---

Bring the stack up locally and confirm the gateway serves traffic.

:::info Testing policy
This project **never uses CI to run tests**. CI runs build, lint, typecheck,
format and packaging gates only. All testing is local integration testing
against a locally-composed stack. If a workflow is the only way to verify
something, that is a finding to report — not a reason to reach for CI.
:::

## Prerequisites

- A recent stable Rust toolchain
- Docker with Compose
- `pnpm` for the admin UI and this documentation site

## Compile the workspace

```bash
cargo check --workspace
```

The quality gates CI enforces, which you can run locally:

```bash
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
cargo fmt --check --all
```

These are hard gates, not style preferences. Library crates additionally forbid
`unwrap()` and `expect()` — use `thiserror` in libraries and reserve `anyhow`
for binary edges.

## Bring up the stack

```bash
docker compose up -d
```

This starts the gateway with its dependencies: Postgres, Iggy, Keto, and
flint-gate. The gateway serves HTTP on the published port and gRPC alongside it.

```bash
curl http://127.0.0.1:28080/healthz
```

:::tip Use `127.0.0.1`, not `localhost`
On macOS, `localhost` resolves to `::1` first. Some container runtimes reset the
IPv6 forward while serving IPv4 normally, which makes a perfectly healthy
gateway look dead. This exact issue cost this project several phases of
misdiagnosis — the gateway logged that it was listening while every request from
the host failed.
:::

## Configuration that will stop you booting

Two variables are mandatory in a production build, and the gateway **refuses to
start** without them rather than starting insecurely:

| Variable | Why it is mandatory |
|---|---|
| `JWT_ISSUER` | Without an issuer check, any token validating against the JWKS is accepted — including one from a different issuer sharing a key source. |
| `GATEWAY_JWKS_URL` | The public keys used to verify inbound JWTs. |

Federation is stricter still: setting `FEDERATION_ENABLED` without both
`FEDERATION_TENANT_ID` and `FEDERATION_CHANNEL_ID` aborts the boot, because
half-configured federation is a data-leak shape.

## Selecting planes

Planes are Cargo features. A deployment compiles only what it runs:

```bash
cargo run -p frf-gateway --features media,federation
```

`SFU_MODE` selects the media path and defaults to `hosted`. Any unrecognised
value also falls back to `hosted` — see [the planes](../theory/planes.md) for
what the sovereign path has and has not been proven to do.

## Next

- [Writing an adapter](writing-an-adapter.md)
- [Authorization](authorization.md)
- [Ports and adapters](../theory/ports.md)
