#!/usr/bin/env bash
# seed-media-view.sh (p24-c003) — grant a subject `view` on a media room in Keto.
#
# The sovereign SFU authorizes media room-join with a Keto `check(subject, "view", room)`
# (ADR-007, fail-closed). For a live decode run (phase-24 c004) the harness subject needs a
# real `view` grant on the test room — this writes that relation tuple. It is a genuine grant,
# NOT a bypass of the check.
#
# The subject is the authenticated identity (JWT `sub`) the gateway stamps on the signal
# envelope (p24-c003), so the grant is stable and predictable (unlike the old ephemeral
# session id).
#
# Usage:
#   E2E_SUBJECT=user:alice E2E_ROOM=e2e-decode-room ./scripts/seed-media-view.sh
# Optional:
#   KETO_WRITE_URL   (default http://localhost:4467)   Keto write API v0.12 uses /admin/relation-tuples (compose maps 4467)
#   KETO_NAMESPACE   (default default)
set -euo pipefail

SUBJECT="${E2E_SUBJECT:?set E2E_SUBJECT (the JWT subject, e.g. user:alice)}"
ROOM="${E2E_ROOM:?set E2E_ROOM (the media room id)}"
KETO_WRITE_URL="${KETO_WRITE_URL:-http://localhost:4467}"
NAMESPACE="${KETO_NAMESPACE:-default}"

echo "[seed-media-view] granting view: subject=${SUBJECT} object=${ROOM} namespace=${NAMESPACE}"

# Keto write API: PUT /relation-tuples creates the tuple (idempotent for the same tuple).
http_status="$(
  curl -sS -o /tmp/seed-media-view.out -w '%{http_code}' \
    -X PUT "${KETO_WRITE_URL}/admin/relation-tuples" \
    -H 'Content-Type: application/json' \
    -d "$(printf '{"namespace":"%s","object":"%s","relation":"view","subject_id":"%s"}' \
          "${NAMESPACE}" "${ROOM}" "${SUBJECT}")"
)"

case "${http_status}" in
  2*) echo "[seed-media-view] OK (HTTP ${http_status}) — ${SUBJECT} may now view ${ROOM}" ;;
  *)
    echo "[seed-media-view] FAILED (HTTP ${http_status})" >&2
    cat /tmp/seed-media-view.out >&2 || true
    exit 1
    ;;
esac
