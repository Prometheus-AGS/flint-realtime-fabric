# Acceptance B — fail-closed runner and ownership-safe cleanup

Date: 2026-09-15
Repository: `flint-realtime-fabric`
Base commit: `c5b184535e95e61f3a3ca327d648ced8980cda3c`

## Failure probes

Each probe ran locally and exited nonzero:

| Probe | Exit | Diagnostic |
|---|---:|---|
| Required command absent from controlled `PATH` | 1 | `missing required command: cargo` |
| `FRF_REQUIRED_SCENARIOS=0` | 1 | `required scenario count must be a positive integer` |
| Required test filter changed to an absent test | 1 | `required test filter cannot be changed or skipped` |

The runner additionally parses the Cargo test summary and requires exactly
`1 passed; 0 failed; 0 ignored`, preventing a zero-test or ignored-test success.

## Cleanup probe

After the successful run, Docker label queries for
`com.docker.compose.project=frf-pri-20260915084331-29908` returned no containers
and no volumes. The receipt records `owned-resources-removed`. Cleanup first
validates the `frf-pri-` ownership prefix and calls Compose with the exact
generated project name; it does not enumerate or remove unrelated resources.

The runner generated random Iggy root credentials for the fresh data volume.
The disposable JWT, private key and JWKS lived under a `mktemp` directory. All
credentials were discarded after cleanup; the receipt stores no username,
password, bearer token or private key.

Docker allocates the loopback host port atomically. The runner discovers it
with `docker compose port` and requires an authenticated Iggy `me` protocol
request to succeed before launching the Rust scenarios.
