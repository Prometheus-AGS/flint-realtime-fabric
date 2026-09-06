# ADR-004: Admin-UI Interactive Login — IdP Path

## Status

Proposed — 2026-07-07 (p19-c004)

Blocks the implementation of a real interactive admin-UI login (phase-18 G1 / phase-19
G4.2). The hardened token gate (ADR-adjacent, p18-c005) is the interim until this is
Accepted and built.

## Scope and reconciliation — 2026-09-06

This remains a proposal for the standalone fabric operator admin UI. Its
compose inventory and interim token gate describe the dated standalone context,
not the integrated ASO deployment. ASO uses Kratos browser/native sessions and
Gate's downstream JWT bridge under [ADR-009](adr-009-aso-runtime-integration.md).
Hydra/OIDC is not an ASO prerequisite, and a pasted token gate is not its patient
workflow login design. This note does not accept the standalone OIDC proposal.

## Context

The admin UI authenticates by having the operator **paste a JWT** into a token gate
(hardened in p18-c005: `exp`-aware, auto-logout on expiry, clears the token on a gateway
`Unauthenticated` response). It is explicitly *not* an interactive login. A real
authorization-code login needs an **authorization server** (an IdP that renders a login
page, authenticates the user, and redirects back with a code). Two facts about the current
stack make this a blocked decision rather than a codeable task:

1. **No IdP is deployed.** `compose.yml` runs `gateway, iggy-server, keto(+migrate),
   flint-gate, surrealdb, postgres`. There is **no Kratos, no Hydra** — nothing that can
   render a login page or run an authorization-code flow. (Keto is Zanzibar authorization,
   not authentication.)
2. **flint-gate is not an authorization server.** flint-gate exposes only
   `POST /oauth/token` (RFC 8693 token-exchange + `client_credentials`) and
   `POST /oauth/introspect`. Its own config states it has **no `authorize` hook**. It mints
   and enforces tokens for streaming/agent workloads; it has **no `/authorize`, no
   `/callback`, no login UI** — it cannot drive an interactive browser login.

Additionally, the admin UI uses a hand-rolled **hash router** (`window.location.hash` in
`App.tsx`); an OIDC redirect returns to a real **path** (`/callback`), so G4.2 also requires
a router change, not just an auth flow.

Per the phase-16–18 discipline (no "healthy but does nothing"), we will not build a
frontend OIDC flow against an authorization server that does not exist. This ADR names the
paths, recommends one, and keeps the token gate as the honest interim.

## Decision

### Options

**Option A — Ory Kratos + Hydra in compose (recommended).**
Add Ory **Hydra** (OAuth2/OIDC authorization server) fronted by Ory **Kratos** (identity /
login-password UI) to the compose stack. The admin UI runs a standard
authorization-code + PKCE flow against Hydra; the gateway verifies the resulting JWT
exactly as it verifies pasted tokens today (`GATEWAY_JWKS_URL` + `JWT_ISSUER` — already
wired). Keto (already present) continues to do authorization.

- **Pros:** standards-compliant OIDC; same Ory family already in the stack (Keto);
  gateway JWT verification is unchanged; Kratos gives real user management (self-service,
  MFA) for free; the flint-gate memory note (`feedback_no-oathkeeper`) is respected —
  this is Kratos/Hydra, **not** Oathkeeper.
- **Cons:** two new services + a database schema to operate; compose/CI weight; Hydra
  client registration + consent config.

**Option B — a flint-gate authorization-code endpoint.**
Extend flint-gate with `/authorize` + `/callback` (and a login surface), turning it into a
minimal authorization server in addition to its token-metering role.

- **Pros:** one fewer moving part in compose; keeps auth concentrated at the gate.
- **Cons:** flint-gate is deliberately **not** an authorization server (no `authorize`
  hook by design); building a compliant OIDC AS + a login UI + user store into it is a
  large scope-creep against its purpose, and re-implements what Hydra/Kratos already do.
  Higher long-term maintenance and security surface for a bespoke AS.

### Recommendation

**Option A (Kratos + Hydra in compose).** It reuses the Ory family already in the stack,
leaves the gateway's JWT verification untouched, and avoids turning flint-gate into a
bespoke authorization server it was explicitly not designed to be. Option B is only
preferable if adding two services to compose is a hard operational constraint — in which
case revisit.

### Interim (until this ADR is Accepted and built)

The **p18-c005 hardened token gate stays** as the admin-UI auth mechanism: operator pastes
a verified JWT; `exp`-aware; auto-logout; clears on `Unauthenticated`. It is an
authenticated-operator tool behind the operator's own access control (documented in
`docs/SECURITY.md` §6).

## Consequences

- **G4.2 is unblocked only after this ADR is Accepted** and the chosen IdP is stood up.
  The frontend work (authorization-code + PKCE, a real `/callback` **path** route → token
  exchange → session, refresh + logout) then follows, including the hash-router → path
  change.
- No admin-UI OIDC code is written before the IdP exists (avoids a "login" that can't work).
- If Option A: gateway config already supports the verification side (`GATEWAY_JWKS_URL`,
  `JWT_ISSUER`); the new work is compose (Hydra + Kratos), Hydra client registration, and
  the frontend flow.

## Related

- p18-c005 — hardened admin-UI token gate (the interim).
- `docs/SECURITY.md` §6 — admin-ui login status (deferred until an IdP exists).
- Memory `feedback_no-oathkeeper` — never use Oathkeeper; flint-gate replaces it. Option A
  is Kratos/Hydra (identity + AS), which does not reintroduce Oathkeeper.
- ADR-003 — toolchain versions (Connect transport the admin UI already uses).
