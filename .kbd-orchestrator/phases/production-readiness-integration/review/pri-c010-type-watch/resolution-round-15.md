# c010 adversarial review round 15 resolution

## Critical: base Compose requires a watch key when CDC is disabled

Disposition: fixed. The base Compose file now gives
`ENTITY_WATCH_CHECKPOINT_KEY` an empty default. Gateway startup remains the
authority for requiring a valid key when CDC and entity watch are enabled, and
the production full-profile renderer still requires the key before producing a
deployment artifact. The deployment-profile regression renders the base
Compose file with `FRF_ENTITY_WATCH_CHECKPOINT_KEY` deliberately unset and
passes, while the full profile continues to receive an explicit generated key.

## Critical: tenant integrity check allegedly rejects other-tenant events

Disposition: rejected after verification. The adapter compares the tenant on
the broker envelope with the tenant inside the decoded mutation. It does not
compare either value with a subscriber tenant. Valid events for every tenant
continue to the application layer, where scope and object authorization run.
The code now names this as an envelope-integrity check and focused regressions
prove that two valid events from different tenants decode successfully while
only an internally inconsistent envelope fails closed.

Strict affected-crate Clippy, both adapter regressions, the repository file-size
gate, and the complete local deployment-profile suite pass.
