#!/usr/bin/env bash
set -euo pipefail

die() { echo "render-profile: $*" >&2; exit 1; }
image=${FRF_SHAPE_GATEWAY_IMAGE:-}
host=${FRF_PUBLIC_HOST:-}
secret=${FRF_TLS_SECRET_NAME:-}
[[ $image =~ ^[A-Za-z0-9._:/-]+@sha256:[0-9a-f]{64}$ ]] ||
  die "FRF_SHAPE_GATEWAY_IMAGE must be an immutable image digest"
[[ $image != *@sha256:0000000000000000000000000000000000000000000000000000000000000000 ]] ||
  die "FRF_SHAPE_GATEWAY_IMAGE cannot use the zero digest"
[[ $host =~ ^[A-Za-z0-9.-]+$ && $host != *.invalid ]] ||
  die "FRF_PUBLIC_HOST must be a DNS hostname outside .invalid"
[[ $secret =~ ^[a-z0-9]([-a-z0-9]*[a-z0-9])?$ ]] ||
  die "FRF_TLS_SECRET_NAME must be a Kubernetes DNS label"
for name in FRF_TLS_CERT_FILE FRF_TLS_KEY_FILE FRF_TLS_CA_FILE; do
  [[ -f ${!name:-} ]] || die "$name must name a readable file"
done
openssl x509 -in "$FRF_TLS_CERT_FILE" -noout >/dev/null 2>&1 ||
  die "FRF_TLS_CERT_FILE is not a valid PEM certificate"
openssl pkey -in "$FRF_TLS_KEY_FILE" -noout >/dev/null 2>&1 ||
  die "FRF_TLS_KEY_FILE is not a valid PEM private key"
openssl verify -CAfile "$FRF_TLS_CA_FILE" "$FRF_TLS_CERT_FILE" >/dev/null 2>&1 ||
  die "FRF_TLS_CERT_FILE is not trusted by FRF_TLS_CA_FILE"
cert_key=$(openssl x509 -in "$FRF_TLS_CERT_FILE" -pubkey -noout |
  openssl pkey -pubin -outform DER 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
private_key=$(openssl pkey -in "$FRF_TLS_KEY_FILE" -pubout 2>/dev/null |
  openssl pkey -pubin -outform DER 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
[[ $cert_key == "$private_key" ]] ||
  die "FRF_TLS_CERT_FILE and FRF_TLS_KEY_FILE do not match"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp -R "$(dirname "$0")/." "$work/"
escaped_image=${image//\//\\/}
sed -i.bak "s/ghcr.io\\/prometheus-ags\\/flint-realtime-fabric-shape@sha256:[0-9a-f]\\{64\\}/$escaped_image/" "$work/gateway.yaml"
sed -i.bak "s/fabric.invalid/$host/g" "$work/gateway.yaml" "$work/ingress.yaml"
sed -i.bak "s/frf-shape-tls/$secret/g" "$work/ingress.yaml"
find "$work" -name '*.bak' -delete
kubectl kustomize "$work"
echo '---'
kubectl create secret generic "$secret" --namespace ssr \
  --type=kubernetes.io/tls \
  --from-file="tls.crt=$FRF_TLS_CERT_FILE" \
  --from-file="tls.key=$FRF_TLS_KEY_FILE" \
  --from-file="ca.crt=$FRF_TLS_CA_FILE" \
  --dry-run=client -o yaml
