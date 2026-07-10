#!/usr/bin/env bash
# run-media-decode.sh (p25-c001) — the authenticated live decoded-media proof runner.
#
# Boots the sovereign SFU stack, authenticates the harness with a REAL RS256 JWT the gateway
# verifier accepts (frf-identity-ory is RS256+JWKS, not HS256 — so we mint RS256 and self-serve
# the matching JWKS), seeds the ADR-007 `view` grant for the token's subject, and runs the
# browser decode harness asserting getStats().framesDecoded > 0.
#
# This exercises the c003 authenticated-subject authz end to end — not a DEV_NO_AUTH bypass.
#
# Prerequisites: docker (running), node, python3, pnpm + Playwright Chromium (admin-ui).
#
# ONE-TIME SETUP before the first run (p31-c001) — the gateway image is a heavy multi-stage build;
# building it IN-RUN OOM-crashed the Colima VM (phase-30), so this runner uses a PRE-BUILT image and
# fails fast if it is absent. Build it once, out-of-band, on a VM with enough memory:
#   colima start --memory 8                                             # restart / size the VM
#   docker compose -f compose.yml -f compose.sovereign.yml build gateway  # build the image ONCE
# Then run this script. (Or run once with PREBUILD_GATEWAY=1 to build-then-run in a single command.)
#
# Env:
#   E2E_SUBJECT       JWT subject to mint + grant view (default dev-integration-user)
#   E2E_ROOM          media room id (default e2e-decode-room)
#   E2E_TENANT_ID     tenant uuid (default …0001, matches the compose CDC tenant)
#   GATEWAY_URL       default http://localhost:28080
#   JWKS_PORT         host port to serve the JWKS on (default 8791)
#   PREBUILD_GATEWAY  set to 1 to build the gateway image in-run (fresh checkout); default off
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

E2E_SUBJECT="${E2E_SUBJECT:-dev-integration-user}"
E2E_ROOM="${E2E_ROOM:-e2e-decode-room}"
E2E_TENANT_ID="${E2E_TENANT_ID:-00000000-0000-0000-0000-000000000001}"
JWKS_PORT="${JWKS_PORT:-8791}"
JWT_DIR="$(mktemp -d)"
COMPOSE=(-f compose.yml -f compose.sovereign.yml)

# TURN relay (p34-c001): coturn now runs as a TURN relay (not STUN-only). The credential is env-sourced
# and NEVER committed — a dev default is generated here for the local proof only. TURN_URL points the
# in-network browser at the relay so it gathers a `typ relay` candidate (the routable pair host/srflx
# couldn't form on the bridge — phase-32). str0m accepts `typ relay`.
export TURN_SECRET="${TURN_SECRET:-$(head -c 24 /dev/urandom | base64 | tr -dc 'a-zA-Z0-9')}"
TURN_USERNAME="${TURN_USERNAME:-frf}"
TURN_URL="${TURN_URL:-turn:coturn:3478}"
export TURN_EXTERNAL_IP="${TURN_EXTERNAL_IP:-127.0.0.1}"

# Host address a bridge-network container uses to reach the runner's host (for the host-served JWKS).
# On Linux (incl. GitHub Actions ubuntu-latest, p35-c001) that is the docker0 bridge gateway
# 172.17.0.1; on macOS Docker Desktop it is host.docker.internal. Override with HOST_ADDR.
if [ -z "${HOST_ADDR:-}" ]; then
  if [ "$(uname -s)" = "Linux" ]; then HOST_ADDR="172.17.0.1"; else HOST_ADDR="host.docker.internal"; fi
fi
# p36-c002: MEDIA_ADVERTISE_IP is now hardcoded to "gateway" in compose.sovereign.yml; the service name
# resolves to the container's own Compose-bridge IP via Docker DNS. Do NOT export it here — a shell-env
# export takes precedence over compose file `environment:` values, which is why run 29069204711 still
# got 172.17.0.1 (the Linux HOST_ADDR) even after compose.sovereign.yml was fixed in c001.
# TURN_EXTERNAL_IP is also no longer used — coturn's --external-ip now expands $(hostname -i) at
# container startup (compose.sovereign.yml entrypoint override, p36-c001), so this export is dead.
# Keep HOST_ADDR for the JWKS URL only (the Python server runs on the host; containers reach it via
# 172.17.0.1 on Linux and host.docker.internal on macOS).

# HOST_NET=1 (p33-c001, Target A): put gateway+browser+caddy+coturn on the Colima VM's host network so
# their ICE candidates share one routable stack (phase-32 had no routable pair on the bridge). Under
# host-net the services are NOT published — the macOS runner reaches the VM gateway at the VM IP, and
# the VM gateway reaches the macOS-served JWKS at the host-side NAT IP. Derive both from Colima's
# default route (VM src = VM IP; VM gateway = host-side). Bridge mode (default) is unchanged.
if [ "${HOST_NET:-0}" = "1" ]; then
  COMPOSE+=(-f compose.host-net.yml)
  _route="$(colima ssh -- ip route get 1 2>/dev/null || true)"
  VM_IP="${VM_IP:-$(printf '%s' "$_route" | sed -n 's/.* src \([0-9.]*\).*/\1/p')}"
  HOST_NAT_IP="${HOST_NAT_IP:-$(printf '%s' "$_route" | sed -n 's/.* via \([0-9.]*\).*/\1/p')}"
  VM_IP="${VM_IP:-192.168.5.1}"
  HOST_NAT_IP="${HOST_NAT_IP:-192.168.5.2}"
  # macOS runner → VM gateway (host-net binds the VM's 8080 directly, not a published port).
  GATEWAY_URL="${GATEWAY_URL:-http://${VM_IP}:8080}"
  # VM gateway → macOS-served JWKS (reach the host across the VM NAT).
  HOST_NET_JWKS_URL="http://${HOST_NAT_IP}:${JWKS_PORT}/jwks.json"
  echo "[run-media-decode] HOST_NET: VM_IP=${VM_IP} HOST_NAT_IP=${HOST_NAT_IP}"
fi
GATEWAY_URL="${GATEWAY_URL:-http://localhost:28080}"

JWKS_PID=""
cleanup() {
  [ -n "$JWKS_PID" ] && kill "$JWKS_PID" 2>/dev/null || true
  # Also kill by port — $JWKS_PID is the subshell; the python child can outlive it and leak the
  # port, silently 404ing the next run.
  pkill -f "http.server ${JWKS_PORT}" 2>/dev/null || true
  docker compose "${COMPOSE[@]}" down -v >/dev/null 2>&1 || true
  rm -rf "$JWT_DIR" || true
}
trap cleanup EXIT

echo "[run-media-decode] minting an RS256 JWT + JWKS (gateway verifier is RS256/JWKS)…"
E2E_JWT="$(node scripts/mint-e2e-jwt.mjs --out-dir "$JWT_DIR" \
  --sub "$E2E_SUBJECT" --aud frf-gateway --tenant "$E2E_TENANT_ID")"

echo "[run-media-decode] serving the JWKS on :${JWKS_PORT} (gateway reaches it via host.docker.internal)…"
# Kill any stale server holding the port (a leaked http.server from a prior run serves a deleted
# dir and 404s our jwks.json — a silent, confusing failure). Then start ours from the JWT dir.
pkill -f "http.server ${JWKS_PORT}" 2>/dev/null || true
sleep 1
( cd "$JWT_DIR" && python3 -m http.server "$JWKS_PORT" >/dev/null 2>&1 ) &
JWKS_PID=$!
# Wait for the server to bind AND actually serve *our* jwks.json (not a stale 404).
for _ in $(seq 1 20); do
  if curl -fsS "http://localhost:${JWKS_PORT}/jwks.json" | grep -q '"keys"'; then
    echo "[run-media-decode] JWKS reachable."
    break
  fi
  sleep 0.5
done
curl -fsS "http://localhost:${JWKS_PORT}/jwks.json" 2>/dev/null | grep -q '"keys"' || {
  echo "[run-media-decode] JWKS server did not serve our keys on :${JWKS_PORT}" >&2; exit 1;
}

# Use a PRE-BUILT gateway image; never build it in-run (p31-c001). The gateway image is a heavy
# multi-stage build (node ui-builder Vite/`tsc` + rust builder) — building it inside the decode run,
# concurrent with the stack + Playwright, OOM-crashed the Colima VM in phase-30. Build it ONCE
# out-of-band, then this runner just brings it up. The image is the compose project's implicit name.
GATEWAY_IMAGE="flint-realtime-fabric-gateway"
if [ "${PREBUILD_GATEWAY:-0}" = "1" ]; then
  echo "[run-media-decode] PREBUILD_GATEWAY=1 — building the gateway image now (fresh checkout)…"
  docker compose "${COMPOSE[@]}" build gateway
elif ! docker image inspect "$GATEWAY_IMAGE" >/dev/null 2>&1; then
  echo "[run-media-decode] gateway image '${GATEWAY_IMAGE}' is absent." >&2
  echo "[run-media-decode] Build it ONCE out-of-band (it is heavy — do NOT build it inside the run):" >&2
  echo "    docker compose -f compose.yml -f compose.sovereign.yml build gateway" >&2
  echo "[run-media-decode] (or re-run with PREBUILD_GATEWAY=1 to build-then-run). On a fresh/low-mem" >&2
  echo "[run-media-decode]  VM also: colima start --memory 8. Then re-run this script." >&2
  exit 1
else
  echo "[run-media-decode] using pre-built gateway image '${GATEWAY_IMAGE}' (no in-run build)."
fi

echo "[run-media-decode] MEDIA_ADVERTISE_IP=${MEDIA_ADVERTISE_IP:-<unset>} TURN_EXTERNAL_IP=${TURN_EXTERNAL_IP:-<unset>}"
echo "[run-media-decode] bringing up the gateway + its deps + coturn + caddy (GATEWAY_JWKS_URL → host JWKS)…"
# coturn (p29-c002) gives the browser a routable srflx candidate the shared-socket SFU accepts
# (phase-28 B1). It has no build step, so it comes up with the gateway.
# Under HOST_NET (p33-c001) the gateway reaches the host-served JWKS across the VM NAT, not via
# host.docker.internal (which does not resolve under network_mode:host).
# p36-c002: set RUST_LOG=debug so str0m ICE candidate lines appear in the gateway.log artifact.
GATEWAY_JWKS_URL="${HOST_NET_JWKS_URL:-http://${HOST_ADDR}:${JWKS_PORT}/jwks.json}" \
  RUST_LOG="debug" \
  docker compose "${COMPOSE[@]}" up -d --no-build gateway coturn caddy playwright

echo "[run-media-decode] waiting for gateway /healthz at ${GATEWAY_URL}…"
for _ in $(seq 1 60); do
  curl -fsS "${GATEWAY_URL}/healthz" >/dev/null 2>&1 && { echo "[run-media-decode] gateway healthy."; break; }
  sleep 2
done
curl -fsS "${GATEWAY_URL}/healthz" >/dev/null 2>&1 || {
  echo "[run-media-decode] gateway never became healthy" >&2
  docker compose "${COMPOSE[@]}" logs --tail 50 gateway >&2 || true
  exit 1
}

echo "[run-media-decode] seeding the media view grant for ${E2E_SUBJECT}…"
E2E_SUBJECT="${E2E_SUBJECT}" E2E_ROOM="${E2E_ROOM}" ./scripts/seed-media-view.sh

echo "[run-media-decode] running the decode harness IN-NETWORK with the authenticated JWT…"
# The browser runs INSIDE the compose network (p30-c001) so its ICE candidates share the SFU's
# address space (Colima VM bridge). The whole repo is mounted (p30-c002) and the host already has a
# complete pnpm-workspace install, so the container reuses `admin-ui/node_modules` directly — no
# in-container install (workspace:* deps only resolve from the repo root). Run the mounted Playwright
# CLI; it reaches the gateway + coturn by service name (GATEWAY_URL/STUN_URL set on the service).
# Don't let a harness failure fire the cleanup trap (down -v) before we capture the gateway's str0m
# lifecycle logs (p28-c001) — grab the exit code, dump logs on failure, then propagate.
harness_rc=0
docker compose "${COMPOSE[@]}" exec -T \
  -e SKIP_INTEGRATION=false \
  -e E2E_TENANT_ID="${E2E_TENANT_ID}" \
  -e E2E_JWT="${E2E_JWT}" \
  -e DECODE_ONLY=1 \
  -e TURN_URL="${TURN_URL}" \
  -e TURN_USERNAME="${TURN_USERNAME}" \
  -e TURN_CREDENTIAL="${TURN_SECRET}" \
  playwright node_modules/.bin/playwright test media-decode --project=chromium || harness_rc=$?

if [ "$harness_rc" -ne 0 ]; then
  echo "[run-media-decode] harness FAILED (rc=${harness_rc}) — capturing gateway logs before teardown…" >&2
  # p36-c002: capture gateway + coturn logs (coturn confirms --external-ip resolved; gateway
  # shows the str0m advertised= candidate). Write to gateway-capture.log — a distinct name so
  # the workflow's "Collect gateway logs" step cannot overwrite it (that step runs after `down -v`
  # removes the containers and produces an empty gateway.log, which was silently clobbering our
  # copy in runs 29069204711 and 29092400906 because both wrote to gateway.log at the same path).
  docker compose "${COMPOSE[@]}" logs --no-color --tail 2000 gateway coturn > /tmp/p29-gateway.log 2>&1 || true
  cp /tmp/p29-gateway.log "${GITHUB_WORKSPACE:-/tmp}/gateway-capture.log" 2>/dev/null || true
  echo "[run-media-decode] gateway logs → /tmp/p29-gateway.log + ${GITHUB_WORKSPACE:-/tmp}/gateway-capture.log" >&2
  exit "$harness_rc"
fi

echo "[run-media-decode] decode proof PASSED — framesDecoded > 0 observed (authenticated path)."
