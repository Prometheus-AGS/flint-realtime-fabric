# Acceptance A — semantic readiness and fail-closed inputs

Captured: 2026-09-15

## Fresh local readiness harness

`crates/frf-gateway/tests/deployment_profiles.rs` constructs a fresh gateway
state for each test and binds new local dependency sockets. It performs the same
network operations as production readiness: HTTP GET for Keto, JWKS and
Electric; TCP connect for Iggy. No missing-service skip or cached service state
is accepted.

| Profile/scenario | Command | Result |
|---|---|---|
| `full`, Keto + non-empty JWKS + Iggy reachable | `cargo test -p frf-gateway --test deployment_profiles --locked` | PASS; `/readyz` 200 and all required checks true |
| `shape-only`, non-empty JWKS + Electric reachable, no Iggy | `cargo test -p frf-gateway --test deployment_profiles --features shape-facade --locked` | PASS; `/readyz` 200, Electric true, Iggy explicitly disabled |
| `full`, authorization authority unreachable | same default-feature test target | PASS; `/readyz` 503 |
| `full`, logical-replication stream inactive | same feature test target | PASS; `/readyz` 503 until CDC reports active |

The two profile runs passed 3/3 and 4/4 tests respectively. Each dependency
probe crosses a real kernel network socket; the harness supplies deterministic
protocol responses so the result does not depend on an external environment.
The owned c002 integration receipt separately proves authenticated event
delivery through the real Iggy adapter and image.

## Negative startup/readiness cases

| Missing or invalid input | Executable proof | Result |
|---|---|---|
| `JWT_ISSUER` | `production_config_without_jwt_issuer_fails_validation` | PASS; startup rejected |
| full-profile `IGGY_CONNECTION_STRING` | `full_profile_without_broker_fails_fast` | PASS; startup rejected |
| broker endpoint down or malformed | `tcp_probe_fails_when_dependency_is_down`, `tcp_probe_reports_unparseable_connection_string` | PASS; not ready |
| authorization endpoint down | `full_profile_readiness_fails_when_authority_is_unreachable` | PASS; 503 |
| empty signing-key set | `jwks_probe_rejects_empty_key_set` | PASS; not ready despite HTTP 200 |
| valid non-empty signing-key set | `jwks_probe_accepts_non_empty_key_set` | PASS |
| Gate signing key absent or mismatched | profile render matrix | PASS; render rejected |
| Gate key not seeded | `flint-gate-db-init` completion dependency | Gate runtime cannot start |
| CDC task exits or never starts replication | CDC readiness watch | `/readyz` 503 |
| incomplete or cross-profile Electric inputs | config test matrix | PASS; startup rejected |

The targeted config suite passed 13/13, readiness probes passed 5/5, and the
disabled broker test passed 1/1. This evidence certifies deployment readiness
semantics only. CDC correctness, persistence, replay and release-image runtime
qualification remain assigned to their later changes.
