#!/usr/bin/env bash
set -euo pipefail

# Smoke test: verify CDC logical replication slot is active after gateway startup.
# Usage: bash scripts/smoke-cdc.sh
# Requires: compose stack running (docker compose up -d)
#
# The slot is created by deploy/postgres/init.sql and activated when the gateway's
# PostgresCdcConsumer opens a walsender replication connection. active=true in
# pg_replication_slots confirms the gateway is streaming WAL.

TIMEOUT="${CDC_SMOKE_TIMEOUT:-30}"
SLOT_NAME="${CDC_SLOT_NAME:-frf_slot}"
POSTGRES_SERVICE="${POSTGRES_SERVICE:-postgres}"

echo "Waiting for CDC slot '${SLOT_NAME}' to become active (max ${TIMEOUT}s)..."

for i in $(seq 1 "${TIMEOUT}"); do
    active=$(docker compose exec -T "${POSTGRES_SERVICE}" \
        psql -U frf -d frf -tAc \
        "SELECT count(*) FROM pg_replication_slots WHERE slot_name='${SLOT_NAME}' AND active=true" \
        2>/dev/null || echo "0")
    active="${active//[[:space:]]/}"
    if [[ "${active}" == "1" ]]; then
        echo "OK: CDC slot '${SLOT_NAME}' is active after ${i}s"
        exit 0
    fi
    sleep 1
done

echo "FAIL: CDC slot '${SLOT_NAME}' was not active after ${TIMEOUT}s"
echo ""
echo "Gateway logs (last 30 lines):"
docker compose logs --tail=30 gateway || true
echo ""
echo "Replication slots:"
docker compose exec -T "${POSTGRES_SERVICE}" \
    psql -U frf -d frf -c \
    "SELECT slot_name, active, restart_lsn FROM pg_replication_slots;" \
    2>/dev/null || true
exit 1
