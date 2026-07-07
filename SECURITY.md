# Security Policy

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report privately to **security@prometheusags.ai**. Include:

- A description of the issue and its impact.
- Steps to reproduce (or a proof-of-concept).
- Affected components/versions and any relevant configuration.

We aim to acknowledge reports within 3 business days and to provide a remediation
timeline after triage. Please give us a reasonable window to fix and release before
public disclosure (coordinated disclosure).

## Supported versions

This project is pre-1.0. Security fixes land on `main`; there is no long-term-support
branch yet. Pin to a specific commit/tag for reproducible deployments.

## Scope

In scope: the gateway (`frf-gateway`), the auth/authz path (flint-gate integration,
Keto, Cedar), tenant isolation, the SDKs, and the CLI. Out of scope: third-party
services (Ory Keto/Kratos, LiveKit, Postgres, Iggy) — report those to their vendors.

## Security model

For how the fabric authenticates, isolates tenants, and authorizes actions (the design,
not this reporting process), see [`docs/SECURITY.md`](docs/SECURITY.md).

## Hardening checklist for operators

- Set `JWT_ISSUER` so only your IdP's tokens are accepted.
- Never set `DEV_NO_AUTH` in production (the release image cannot honor it anyway).
- Provide all secrets via a secret manager, never the repo (see `docs/ENVIRONMENT.md`).
- Configure `CORS_ALLOWED_ORIGINS`, rate limits, and a TLS-terminating proxy.
- Run `cargo audit` / `cargo deny` in CI to catch vulnerable dependencies.
