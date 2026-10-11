# Acceptance A — authenticated publish reaches the subscribed gateway

Date: 2026-09-15
Repository: `flint-realtime-fabric`
Base commit: `c5b184535e95e61f3a3ca327d648ced8980cda3c`
Receipt: `20260915084331-29908.json`

## Command

```bash
FRF_INTEGRATION_RECEIPT_DIR=.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c002-local-fixtures \
  bash scripts/run-local-integration.sh
```

## Result

- The runner started the unique Compose project `frf-pri-20260915084331-29908`.
- An in-process gateway used the production Axum router with a real
  `IggyBroker`, `OryIdentityVerifier`, RS256 JWKS endpoint and
  `ConfiguredAuthzProvider`.
- The disabled-delivery mutation still completed the authenticated HTTP publish,
  then suppressed the broker subscription stream at the gateway delivery seam.
  It exited `101` with the assertion marker `required delivery missing`.
- Restoring publish produced exactly one passing required test and the marker
  `required delivery received`.
- The final run began with caller environment `FRF_DISABLE_DELIVERY=1`; the
  positive scenario explicitly removed it and still passed.
- Runner exit status: `0`; completed required scenarios: `2/2`.

The receipt binds the compose, test and runner contents by SHA-256 and records
the exact Iggy image ID and repository digest.
