# c005 acceptance A — Gate identity and generic stream authorization

Status: **PASS**

## Runtime boundary

`scripts/run-authority-lifetime.sh` created owned, ephemeral Postgres and Ory
Keto containers, a detached Flint Gate worktree at
`de88f40dbfe77bc03203bafc2b64de9815fe27b2`, an external IdP fixture, and
ephemeral RSA key pairs. Gate validated the three external IdP tokens and minted
the tokens consumed by Fabric. Fabric then verified Gate's asymmetric JWKS and
queried the real Keto read API for channel and object decisions.

The successful runtime receipt is `c005-authority-runtime.json`: exit 0 and seven
completed scenarios. It proves matching issuer/audience/tenant/subject and an
allowed object, wrong issuer and audience rejection, same-tenant subject denial,
cross-tenant filtering, object denial, and observed tuple revocation after the
one-second cache bound.

Two failed fixture attempts were useful integration findings. The selected Keto
version exposes writes at `/admin/relation-tuples` and returns HTTP 403 with
`{"allowed":false}` for a denied check. The adapter and its tests now implement
those real semantics. The passing receipt supersedes the earlier failure receipt
at the same run id.

The post-review run also exposed a cold-build timing flaw in the harness: Gate
tokens were minted before a nine-minute integration-test compile and correctly
expired before use. The runner now compiles the test before starting its owned
services and minting credentials. Every Postgres run also receives a fresh
random password that exists only in the temporary fixture directory and owned
container. The final passing receipt supersedes that failed attempt.

## Focused local checks

- `cargo test --offline -p frf-app --test subscribe_pipeline --locked`: 8 passed.
  Idle token expiry, channel revocation, object denial, authority failure and
  cross-tenant filtering all terminate or suppress output before a protected
  event escapes.
- `cargo test --offline -p frf-identity-ory --locked`: 11 passed. Wrong issuer,
  wrong audience, expired tokens, unknown keys and removal of a cached signing
  key after the bounded JWKS age are covered.
- `cargo test --offline -p frf-authz-keto --locked`: 9 passed. The suite includes
  the invalidation-generation fence that rejects an older in-flight decision
  after revocation.
- `scripts/run-authority-lifetime.sh`: 7 real Gate/Keto/Fabric scenarios passed.

All commands used isolated local Cargo build directories. No CI test execution
was used.
