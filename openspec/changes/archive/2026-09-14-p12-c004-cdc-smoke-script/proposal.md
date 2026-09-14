# p12-c004 — CDC Smoke Script

## Phase
phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Summary

Add `scripts/smoke-cdc.sh` that verifies the CDC consumer successfully
activates its replication slot after the compose stack starts. This is a
developer-facing smoke script (not a Dagger stage) that can be run manually
or from a Makefile target.

## Files to Create

- `scripts/smoke-cdc.sh` (NEW, executable)

```bash
#!/usr/bin/env bash
set -euo pipefail

# Smoke test: verify CDC logical replication slot is active after gateway startup.
# Usage: bash scripts/smoke-cdc.sh
# Requires: docker compose stack running (docker compose up -d)

TIMEOUT=30
SLOT_NAME="${CDC_SLOT_NAME:-frf_slot}"
POSTGRES_SERVICE="${POSTGRES_SERVICE:-postgres}"

echo "Waiting for CDC slot '${SLOT_NAME}' to become active (max ${TIMEOUT}s)..."

for i in $(seq 1 "${TIMEOUT}"); do
    active=$(docker compose exec -T "${POSTGRES_SERVICE}" \
        psql -U frf -d frf -tAc \
        "SELECT count(*) FROM pg_replication_slots WHERE slot_name='${SLOT_NAME}' AND active=true" \
        2>/dev/null || echo "0")
    if [[ "${active}" == "1" ]]; then
        echo "OK: CDC slot '${SLOT_NAME}' is active after ${i}s"
        exit 0
    fi
    sleep 1
done

echo "FAIL: CDC slot '${SLOT_NAME}' was not active after ${TIMEOUT}s"
echo "Gateway logs:"
docker compose logs --tail=20 gateway
exit 1
```

## Design Notes

The script polls `pg_replication_slots` for `active=true` — this is only true
when the gateway's `PostgresCdcConsumer` has opened a walsender connection and
is streaming WAL. It polls for up to 30 seconds to allow for gateway startup
time (Iggy connection, channel pre-creation, then CDC connection).

The script is `docker compose exec -T` (non-interactive) so it works in CI
without a TTY.

**Why not a Dagger stage?** The CDC smoke requires a running compose stack with
a mounted Docker socket — the same constraint as Stage 10. Since Stage 10 is
already opt-in, adding another opt-in stage adds no value. The shell script
can be called inside Stage 10 as an additional verification step in a future
phase.

## Exit Criteria

- `scripts/smoke-cdc.sh` is executable (`chmod +x`)
- Running `bash scripts/smoke-cdc.sh` against a live compose stack exits 0
- Running against a stack without the gateway exits 1 with a useful error message
