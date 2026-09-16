#!/usr/bin/env bash
set -euo pipefail

die() { echo "test-deployment-profiles: $*" >&2; exit 1; }
expect_failure() {
  local label=$1
  shift
  if "$@" >/dev/null 2>&1; then
    die "$label unexpectedly succeeded"
  fi
  echo "PASS negative: $label"
}

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"
work=$(mktemp -d)
suffix=$$
network="frf-profile-test-$suffix"
backend="frf-profile-backend-$suffix"
edge="frf-profile-edge-$suffix"
shape_edge="frf-profile-shape-edge-$suffix"

cleanup() {
  docker rm -f "$shape_edge" "$edge" "$backend" >/dev/null 2>&1 || true
  docker network rm "$network" >/dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT

cat >"$work/cert.cnf" <<'EOF'
[req]
distinguished_name = subject
x509_extensions = extensions
prompt = no
[subject]
CN = localhost
[extensions]
subjectAltName = DNS:localhost,IP:127.0.0.1
basicConstraints = critical,CA:TRUE
keyUsage = critical,keyCertSign,digitalSignature,keyEncipherment
EOF
openssl req -x509 -newkey rsa:2048 -nodes -days 1 -config "$work/cert.cnf" \
  -keyout "$work/tls.key" -out "$work/tls.crt" >/dev/null 2>&1
cp "$work/tls.crt" "$work/ca.crt"
openssl genpkey -algorithm RSA -out "$work/wrong.key" -pkeyopt rsa_keygen_bits:2048 \
  >/dev/null 2>&1
openssl genpkey -algorithm RSA -out "$work/gate-private.pem" -pkeyopt rsa_keygen_bits:2048 \
  >/dev/null 2>&1
openssl pkey -in "$work/gate-private.pem" -pubout -out "$work/gate-public.pem" \
  >/dev/null 2>&1

cat >"$work/gate.yaml" <<'EOF'
server:
  listen: 0.0.0.0:4456
  admin_listen: 0.0.0.0:4457
auth_providers:
  production_identity:
    type: jwt
    jwks_url: https://identity.test/.well-known/jwks.json
    issuer: https://identity.test
jwt:
  signing_algorithm: RS256
EOF
cat >"$work/dev-gate.yaml" <<'EOF'
auth_providers:
  bypass:
    type: anonymous
EOF
cat >"$work/symmetric-gate.yaml" <<'EOF'
auth_providers:
  production_identity:
    type: jwt
    jwks_url: https://identity.test/.well-known/jwks.json
    issuer: https://identity.test
jwt:
  signing_algorithm: HS256
EOF
echo '{}' >"$work/catalog.json"

digest() { printf '%s@sha256:%064d' "$1" "$2"; }
export FRF_TLS_PROXY_IMAGE="nginx@sha256:42a516af16b852e33b7682d5ef8acbd5d13fe08fecadc7ed98605ba5e3b26ab8"
export FLINT_GATE_IMAGE
FLINT_GATE_IMAGE=$(digest example/flint-gate 2)
export FRF_POSTGRES_IMAGE
FRF_POSTGRES_IMAGE=$(digest postgres 3)
export FRF_FULL_GATEWAY_IMAGE
FRF_FULL_GATEWAY_IMAGE=$(digest example/frf 4)
export FRF_SHAPE_GATEWAY_IMAGE
FRF_SHAPE_GATEWAY_IMAGE=$(digest example/frf-shape 5)
export FRF_IGGY_IMAGE
FRF_IGGY_IMAGE=$(digest iggyrs/iggy 6)
export FRF_SURREAL_IMAGE
FRF_SURREAL_IMAGE=$(digest surrealdb/surrealdb 9)
export FRF_KETO_IMAGE
FRF_KETO_IMAGE=$(digest oryd/keto 7)
export FRF_ELECTRIC_IMAGE
FRF_ELECTRIC_IMAGE=$(digest electricsql/electric 8)
export FRF_JWT_AUDIENCE=frf-gateway
export FRF_JWT_ISSUER=https://identity.test
export FRF_PUBLIC_ORIGIN=https://localhost
export FRF_POSTGRES_USER=frf FRF_POSTGRES_PASSWORD=secret FRF_POSTGRES_DB=frf
export FLINT_GATE_CONFIG_FILE="$work/gate.yaml"
export FLINT_GATE_JWT_PRIVATE_KEY_FILE="$work/gate-private.pem"
export FLINT_GATE_JWT_PUBLIC_KEY_FILE="$work/gate-public.pem"
export FRF_TLS_CERT_FILE="$work/tls.crt" FRF_TLS_KEY_FILE="$work/tls.key"
export FRF_TLS_CA_FILE="$work/ca.crt"
export FRF_IGGY_USERNAME=frf FRF_IGGY_PASSWORD=secret
export FRF_SURREAL_USERNAME=frf FRF_SURREAL_PASSWORD=secret
export FRF_SURREAL_NAMESPACE=flint FRF_SURREAL_DATABASE=entity_projection
FRF_ENTITY_WATCH_CHECKPOINT_KEY=$(openssl rand -hex 32)
export FRF_ENTITY_WATCH_CHECKPOINT_KEY
export FRF_CDC_TENANT_ID=00000000-0000-0000-0000-000000000001
export FRF_SHAPE_CATALOG_FILE="$work/catalog.json"

env -u FRF_ENTITY_WATCH_CHECKPOINT_KEY docker compose -f compose.yml config >/dev/null
echo "PASS base compose: entity watch key is optional when CDC is disabled"

scripts/render-deployment-profile.sh full "$work/full.yml"
scripts/render-deployment-profile.sh shape-only "$work/shape.yml"
for profile in full shape; do
  docker compose -f "$work/$profile.yml" config --format json >"$work/$profile.json"
  jq -e '[.services | to_entries[] | select((.value.ports // []) | length > 0) | .key] == ["edge"]' \
    "$work/$profile.json" >/dev/null
  docker compose -f "$work/$profile.yml" config --images |
    while IFS= read -r image; do
      [[ $image =~ @sha256:[0-9a-f]{64}$ ]] || die "$profile has unpinned image $image"
    done
done
jq -e '.services.gateway.environment.GATEWAY_PROFILE == "full" and
  .services.gateway.environment.MEDIA_ENABLED == "false" and
  .services.gateway.environment.AGENT_ENABLED == "false" and
  .services.gateway.environment.ADMIN_ENABLED == "false" and
  (.services.gateway.environment | has("SHAPE_ELECTRIC_URL") | not) and
  .services["flint-gate"].depends_on["flint-gate-db-init"].condition == "service_completed_successfully" and
  .services["flint-gate"].environment.FLINT_GATE_REQUIRE_DATABASE == "true" and
  .services.gateway.depends_on.surreal.condition == "service_healthy" and
  .services.gateway.environment.ENTITY_PROJECTION_URL == "ws://surreal:8000" and
  .services.surreal.volumes[0].target == "/data"' \
  "$work/full.json" >/dev/null
jq -e '.services.gateway.environment.GATEWAY_PROFILE == "shape-only" and
  .services.gateway.environment.GRPC_PORT == "0" and
  .services.gateway.environment.CDC_ENABLED == "false" and
  .services.gateway.environment.FEDERATION_ENABLED == "false" and
  (.services.gateway.environment | has("IGGY_CONNECTION_STRING") | not) and
  .services["flint-gate"].depends_on["flint-gate-db-init"].condition == "service_completed_successfully" and
  .services["flint-gate"].environment.FLINT_GATE_REQUIRE_DATABASE == "true"' \
  "$work/shape.json" >/dev/null
echo "PASS render: immutable images, scoped flags, edge-only published ports"

saved_issuer=$FRF_JWT_ISSUER
unset FRF_JWT_ISSUER
expect_failure "missing issuer" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FRF_JWT_ISSUER=$saved_issuer
saved_iggy=$FRF_IGGY_IMAGE
export FRF_IGGY_IMAGE=iggyrs/iggy:latest
expect_failure "mutable image" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FRF_IGGY_IMAGE=$saved_iggy
saved_key=$FRF_TLS_KEY_FILE
export FRF_TLS_KEY_FILE=$work/missing.key
expect_failure "missing TLS key" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FRF_TLS_KEY_FILE=$work/wrong.key
expect_failure "mismatched TLS key" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FRF_TLS_KEY_FILE=$saved_key
saved_gate=$FLINT_GATE_CONFIG_FILE
export FLINT_GATE_CONFIG_FILE=$work/dev-gate.yaml
expect_failure "development Gate provider" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FLINT_GATE_CONFIG_FILE=$saved_gate
export FLINT_GATE_CONFIG_FILE=$work/symmetric-gate.yaml
expect_failure "symmetric Gate signing algorithm" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FLINT_GATE_CONFIG_FILE=$saved_gate
saved_gate_public=$FLINT_GATE_JWT_PUBLIC_KEY_FILE
export FLINT_GATE_JWT_PUBLIC_KEY_FILE=$work/missing-public.pem
expect_failure "missing Gate public key" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FLINT_GATE_JWT_PUBLIC_KEY_FILE=$saved_gate_public
saved_gate_private=$FLINT_GATE_JWT_PRIVATE_KEY_FILE
export FLINT_GATE_JWT_PRIVATE_KEY_FILE=$work/wrong.key
expect_failure "mismatched Gate signing keys" scripts/render-deployment-profile.sh full "$work/reject.yml"
export FLINT_GATE_JWT_PRIVATE_KEY_FILE=$saved_gate_private

cat >"$work/backend.conf" <<'EOF'
events {}
http { server { listen 8080; location / { return 200 "gateway-ready\n"; } } }
EOF
docker network create "$network" >/dev/null
docker run -d --name "$backend" --network "$network" --network-alias gateway \
  -v "$work/backend.conf:/etc/nginx/nginx.conf:ro" "$FRF_TLS_PROXY_IMAGE" >/dev/null
docker run -d --name "$edge" --network "$network" -p 127.0.0.1::8443 \
  -v "$root/deploy/profiles/nginx-full.conf:/etc/nginx/nginx.conf:ro" \
  -v "$work/tls.crt:/run/tls/tls.crt:ro" -v "$work/tls.key:/run/tls/tls.key:ro" \
  -v "$work/ca.crt:/run/tls/ca.crt:ro" "$FRF_TLS_PROXY_IMAGE" >/dev/null
port=$(docker port "$edge" 8443/tcp | sed 's/.*://')
for _ in {1..20}; do
  if curl --cacert "$work/ca.crt" -fsS "https://localhost:$port/healthz" >"$work/https.out"; then
    break
  fi
  sleep 0.25
done
rg -q '^gateway-ready$' "$work/https.out" || die "valid HTTPS did not reach gateway"
expect_failure "plaintext on TLS edge" curl -fsS "http://127.0.0.1:$port/healthz"
[[ -z $(docker port "$backend" 8080/tcp) ]] || die "backend unexpectedly published a host port"
echo "PASS TLS: trusted HTTPS reached the backend; plaintext and backend bypass were denied"

docker run -d --name "$shape_edge" --network "$network" -p 127.0.0.1::8443 \
  -v "$root/deploy/profiles/nginx-shape-only.conf:/etc/nginx/nginx.conf:ro" \
  -v "$work/tls.crt:/run/tls/tls.crt:ro" -v "$work/tls.key:/run/tls/tls.key:ro" \
  -v "$work/ca.crt:/run/tls/ca.crt:ro" "$FRF_TLS_PROXY_IMAGE" >/dev/null
shape_port=$(docker port "$shape_edge" 8443/tcp | sed 's/.*://')
for _ in {1..20}; do
  if curl --cacert "$work/ca.crt" -fsS "https://localhost:$shape_port/healthz" \
    >"$work/shape-https.out"; then
    break
  fi
  sleep 0.25
done
rg -q '^gateway-ready$' "$work/shape-https.out" || die "shape HTTPS did not reach gateway"
status=$(curl --cacert "$work/ca.crt" -sS -o /dev/null -w '%{http_code}' \
  "https://localhost:$shape_port/ws/v1/subscribe")
[[ $status == 404 ]] || die "shape edge admitted an out-of-profile route with status $status"
echo "PASS shape edge: trusted HTTPS succeeded and an out-of-profile route returned 404"

export FRF_PUBLIC_HOST=workspace-ssr.test
export FRF_TLS_SECRET_NAME=frf-shape-test-tls
k8s/overlays/ssr/render-profile.sh >"$work/ssr.yaml"
kubectl apply --dry-run=client --validate=false -f "$work/ssr.yaml" >/dev/null
rg -q 'name: frf-shape-test-tls' "$work/ssr.yaml" ||
  die "SSR render omitted its bound TLS Secret"
rg -q 'value: https://workspace-ssr.test/flint-gate' "$work/ssr.yaml" ||
  die "SSR render did not bind JWT issuer to its public host"
saved_tls_secret=$FRF_TLS_SECRET_NAME
unset FRF_TLS_SECRET_NAME
expect_failure "missing SSR TLS secret name" k8s/overlays/ssr/render-profile.sh
export FRF_TLS_SECRET_NAME=$saved_tls_secret
saved_tls_key=$FRF_TLS_KEY_FILE
export FRF_TLS_KEY_FILE=$work/wrong.key
expect_failure "mismatched SSR TLS key" k8s/overlays/ssr/render-profile.sh
export FRF_TLS_KEY_FILE=$saved_tls_key
echo "PASS SSR TLS: rendered Secret, host-bound identity and negative inputs"
