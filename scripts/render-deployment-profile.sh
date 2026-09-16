#!/usr/bin/env bash
set -euo pipefail

die() {
  echo "render-deployment-profile: $*" >&2
  exit 1
}

require_var() {
  local name=$1
  [[ -n ${!name:-} ]] || die "$name is required"
}

require_file() {
  local name=$1
  require_var "$name"
  [[ -f ${!name} ]] || die "$name does not name a readable file"
}

require_digest() {
  local name=$1 value
  require_var "$name"
  value=${!name}
  [[ $value =~ ^[A-Za-z0-9._:/-]+@sha256:[0-9a-f]{64}$ ]] ||
    die "$name must be an immutable image reference ending in @sha256:<64 lowercase hex>"
  [[ $value != *@sha256:0000000000000000000000000000000000000000000000000000000000000000 ]] ||
    die "$name cannot use the zero digest"
}

require_https() {
  local name=$1
  require_var "$name"
  [[ ${!name} == https://* ]] || die "$name must use https://"
}

profile=${1:-}
output=${2:--}
case "$profile" in
  full)
    compose_file=compose.yml
    profile_images=(FRF_FULL_GATEWAY_IMAGE FRF_IGGY_IMAGE FRF_KETO_IMAGE FRF_SURREAL_IMAGE)
    profile_vars=(FRF_IGGY_USERNAME FRF_IGGY_PASSWORD FRF_CDC_TENANT_ID \
      FRF_SURREAL_USERNAME FRF_SURREAL_PASSWORD FRF_SURREAL_NAMESPACE \
      FRF_SURREAL_DATABASE)
    ;;
  shape-only)
    compose_file=compose.shape-only.yml
    profile_images=(FRF_SHAPE_GATEWAY_IMAGE FRF_ELECTRIC_IMAGE)
    profile_vars=()
    require_file FRF_SHAPE_CATALOG_FILE
    ;;
  *) die "usage: $0 <full|shape-only> [output-file|-]" ;;
esac

for name in FRF_TLS_PROXY_IMAGE FLINT_GATE_IMAGE FRF_POSTGRES_IMAGE "${profile_images[@]}"; do
  require_digest "$name"
done
for name in FRF_JWT_AUDIENCE FRF_POSTGRES_USER FRF_POSTGRES_PASSWORD \
  FRF_POSTGRES_DB "${profile_vars[@]}"; do
  require_var "$name"
done
require_https FRF_JWT_ISSUER
require_https FRF_PUBLIC_ORIGIN
require_file FLINT_GATE_CONFIG_FILE
require_file FLINT_GATE_JWT_PRIVATE_KEY_FILE
require_file FLINT_GATE_JWT_PUBLIC_KEY_FILE
require_file FRF_TLS_CERT_FILE
require_file FRF_TLS_KEY_FILE
require_file FRF_TLS_CA_FILE

openssl x509 -in "$FRF_TLS_CERT_FILE" -noout >/dev/null 2>&1 ||
  die "FRF_TLS_CERT_FILE is not a valid PEM certificate"
openssl pkey -in "$FRF_TLS_KEY_FILE" -noout >/dev/null 2>&1 ||
  die "FRF_TLS_KEY_FILE is not a valid PEM private key"
openssl verify -CAfile "$FRF_TLS_CA_FILE" "$FRF_TLS_CERT_FILE" >/dev/null 2>&1 ||
  die "FRF_TLS_CERT_FILE is not trusted by FRF_TLS_CA_FILE"

cert_public_key=$(openssl x509 -in "$FRF_TLS_CERT_FILE" -pubkey -noout |
  openssl pkey -pubin -outform DER 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
private_public_key=$(openssl pkey -in "$FRF_TLS_KEY_FILE" -pubout 2>/dev/null |
  openssl pkey -pubin -outform DER 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
[[ $cert_public_key == "$private_public_key" ]] ||
  die "FRF_TLS_CERT_FILE and FRF_TLS_KEY_FILE do not match"

openssl pkey -in "$FLINT_GATE_JWT_PRIVATE_KEY_FILE" -noout >/dev/null 2>&1 ||
  die "FLINT_GATE_JWT_PRIVATE_KEY_FILE is not a valid PEM private key"
openssl pkey -pubin -in "$FLINT_GATE_JWT_PUBLIC_KEY_FILE" -noout >/dev/null 2>&1 ||
  die "FLINT_GATE_JWT_PUBLIC_KEY_FILE is not a valid PEM public key"
gate_private_public_key=$(openssl pkey -in "$FLINT_GATE_JWT_PRIVATE_KEY_FILE" -pubout 2>/dev/null |
  openssl pkey -pubin -outform DER 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
gate_public_key=$(openssl pkey -pubin -in "$FLINT_GATE_JWT_PUBLIC_KEY_FILE" -outform DER 2>/dev/null |
  shasum -a 256 | cut -d' ' -f1)
[[ $gate_private_public_key == "$gate_public_key" ]] ||
  die "FLINT_GATE_JWT_PRIVATE_KEY_FILE and FLINT_GATE_JWT_PUBLIC_KEY_FILE do not match"

if rg -n '/Users/|DEV_NO_AUTH|dev-endpoints' "$compose_file" deploy/profiles >/dev/null; then
  die "profile source contains a developer path or authorization bypass"
fi
if rg -n 'type:[[:space:]]*anonymous|dev_passthrough|DEV_NO_AUTH' \
  "$FLINT_GATE_CONFIG_FILE" >/dev/null; then
  die "FLINT_GATE_CONFIG_FILE contains a development authorization bypass"
fi
if rg -n "^[[:space:]]*signing_algorithm:[[:space:]]*['\"]?HS(256|384|512)" \
  "$FLINT_GATE_CONFIG_FILE" >/dev/null; then
  die "FLINT_GATE_CONFIG_FILE cannot publish verifiable keys with an HMAC signing algorithm"
fi
if ! rg -n "^[[:space:]]*signing_algorithm:[[:space:]]*['\"]?(RS256|RS384|RS512|ES256|ES384)['\"]?[[:space:]]*$" \
  "$FLINT_GATE_CONFIG_FILE" >/dev/null; then
  die "FLINT_GATE_CONFIG_FILE must select an asymmetric JWT signing_algorithm"
fi

rendered=$(mktemp)
trap 'rm -f "$rendered"' EXIT
docker compose -f "$compose_file" config >"$rendered"

if rg -n '^[[:space:]]+build:' "$rendered" >/dev/null; then
  die "rendered profile contains a source build; supply a pinned release image"
fi
while IFS= read -r image; do
  [[ $image =~ ^[A-Za-z0-9._:/-]+@sha256:[0-9a-f]{64}$ ]] ||
    die "rendered profile contains an unpinned image: $image"
done < <(docker compose -f "$compose_file" config --images)

if [[ $output == - ]]; then
  cat "$rendered"
else
  cp "$rendered" "$output"
fi
