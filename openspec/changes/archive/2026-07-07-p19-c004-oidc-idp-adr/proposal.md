# p19-c004 — ADR for the admin-UI OIDC IdP path

## Why

Phase-19 G4 asks for a "full admin-UI OIDC login." The assessment established this is
**blocked on infrastructure that does not exist**: `compose.yml` has no IdP (no Kratos,
no Hydra), and flint-gate is a token-metering proxy with **no `authorize` hook** — it
cannot drive an interactive browser login. Building a frontend OIDC flow against a
non-existent authorization server would be a "healthy but does nothing" login, which the
phase-16–18 discipline forbids. The load-bearing move is a **decision**: which IdP path to
take. This change records that decision as an ADR and re-affirms the token gate as interim.

## What Changes

Documentation only.

1. **`docs/decisions/adr-004-admin-ui-oidc-idp.md`** (new) — Proposed ADR: states the
   constraint (no IdP; flint-gate has no auth-code endpoint; hash-router needs a path route
   for `/callback`), presents **Option A (Kratos + Hydra in compose)** vs. **Option B
   (flint-gate `/authorize` endpoint)**, **recommends Option A**, and defines the interim
   (the p18-c005 token gate) plus the consequences (G4.2 unblocks only after the ADR is
   Accepted + the IdP is stood up).
2. **`docs/SECURITY.md` §6** — point the admin-ui login row at ADR-004 as the decision of
   record for the deferral.

## Non-goals

- Standing up an IdP or writing any OIDC frontend code (blocked until the ADR is Accepted).
- Changing the token gate (it remains the hardened interim).

## Impact

- Affected: `docs/decisions/adr-004-admin-ui-oidc-idp.md` (new), `docs/SECURITY.md`.
- The IdP decision is now explicit and reviewable; G4.2 has a named prerequisite.
