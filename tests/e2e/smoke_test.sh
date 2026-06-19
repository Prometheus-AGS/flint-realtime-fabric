#!/usr/bin/env bash
# E2E smoke test — requires a running frf-gateway.
#
# Usage:
#   ./tests/e2e/smoke_test.sh
#
# Environment variables:
#   FRF_GATEWAY_URL   (default: http://localhost:4000)
#   FRF_CHANNEL_ID    (default: 00000000-0000-0000-0000-000000000001)
#   JWT_TOKEN         bearer token for authenticated requests (optional)
#
# Each subscriber binary exits 0 on receipt; non-zero failures are reported.

set -euo pipefail

GATEWAY="${FRF_GATEWAY_URL:-http://localhost:4000}"
CHANNEL="${FRF_CHANNEL_ID:-00000000-0000-0000-0000-000000000001}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd ../.. && pwd)"
PASS=0
FAIL=0

log() { printf '[smoke] %s\n' "$*"; }
ok()  { log "PASS: $*"; ((PASS+=1)); }
err() { log "FAIL: $*"; ((FAIL+=1)); }

# ── health check ─────────────────────────────────────────────────────────────
log "Health check ${GATEWAY}/healthz"
if curl -sf "${GATEWAY}/healthz" >/dev/null; then
  ok "gateway /healthz"
else
  err "gateway /healthz not reachable — is frf-gateway running?"
  exit 1
fi

# ── publish a test event via HTTP (Connect-RPC / JSON) ───────────────────────
EVENT_ID="$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || echo 'test-event-id')"
PAYLOAD_B64="$(echo -n '{"smoke":true}' | base64)"

log "Publishing test event id=${EVENT_ID}"
PUB_RESPONSE=$(curl -sf -X POST "${GATEWAY}/flint.v1.SpineService/Publish" \
  -H "Content-Type: application/json" \
  ${JWT_TOKEN:+-H "Authorization: Bearer ${JWT_TOKEN}"} \
  -d "{
    \"envelope\": {
      \"id\": \"${EVENT_ID}\",
      \"channel\": {
        \"id\": \"${CHANNEL}\",
        \"tenantId\": \"e2e-tenant\",
        \"path\": \"smoke\"
      },
      \"kind\": 1,
      \"payload\": \"${PAYLOAD_B64}\"
    }
  }") || true

if [ -n "${PUB_RESPONSE}" ]; then
  ok "publish event"
else
  err "publish returned empty or error response"
fi

# ── TS subscriber ─────────────────────────────────────────────────────────────
log "Running TS subscriber…"
if command -v tsx &>/dev/null || (cd "${REPO_ROOT}" && npx --yes tsx --version &>/dev/null 2>&1); then
  export FRF_GATEWAY_URL="${GATEWAY}" FRF_CHANNEL_ID="${CHANNEL}"
  if (cd "${REPO_ROOT}" && npx tsx tests/e2e/ts/smoke.ts); then
    ok "TS SDK smoke"
  else
    err "TS SDK smoke"
  fi
else
  log "SKIP: tsx not available (install with: npm i -g tsx)"
fi

# ── Go subscriber ─────────────────────────────────────────────────────────────
log "Running Go subscriber…"
if command -v go &>/dev/null; then
  export FRF_GATEWAY_URL="${GATEWAY}" FRF_CHANNEL_ID="${CHANNEL}"
  if (cd "${REPO_ROOT}/sdks/go" && go run -tags integration "${REPO_ROOT}/tests/e2e/go/main.go"); then
    ok "Go SDK smoke"
  else
    err "Go SDK smoke"
  fi
else
  log "SKIP: go not found on PATH"
fi

# ── C# subscriber ─────────────────────────────────────────────────────────────
log "Running C# subscriber…"
if command -v dotnet &>/dev/null; then
  export FRF_GATEWAY_URL="${GATEWAY}" FRF_CHANNEL_ID="${CHANNEL}"
  if dotnet run --project "${REPO_ROOT}/tests/e2e/csharp/Smoke.csproj"; then
    ok "C# SDK smoke"
  else
    err "C# SDK smoke"
  fi
else
  log "SKIP: dotnet not found on PATH"
fi

# ── summary ──────────────────────────────────────────────────────────────────
echo ""
log "Results: ${PASS} passed, ${FAIL} failed"
[ "${FAIL}" -eq 0 ]
