#!/usr/bin/env bash
# Run the browser Layer 3 suite against a local, disposable Compose stack.

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$WORKSPACE_ROOT/compose.ci.yml"
GATEWAY_URL="${GATEWAY_URL:-http://127.0.0.1:28080}"

cleanup() {
  docker compose -f "$COMPOSE_FILE" down -v
}
trap cleanup EXIT

cd "$WORKSPACE_ROOT"
docker compose -f "$COMPOSE_FILE" up -d --build

for _ in $(seq 1 60); do
  if curl -fsS "$GATEWAY_URL/healthz" >/dev/null; then
    break
  fi
  sleep 2
done
curl -fsS "$GATEWAY_URL/healthz" >/dev/null

pnpm --dir admin-ui install --frozen-lockfile
pnpm --dir admin-ui exec playwright install chromium

SKIP_INTEGRATION=false \
WASM_AVAILABLE=1 \
GATEWAY_URL="$GATEWAY_URL" \
DEV_ENDPOINTS_ENABLED=true \
pnpm --dir admin-ui exec playwright test e2e/ --reporter=list
