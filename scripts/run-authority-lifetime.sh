#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT_DIR
readonly GATE_REPO="${FLINT_GATE_ROOT:-$ROOT_DIR/../flint-gate}"
readonly GATE_REVISION="${FLINT_GATE_REVISION:-de88f40dbfe77bc03203bafc2b64de9815fe27b2}"
readonly POSTGRES_IMAGE="postgres@sha256:d9c304353c031b21e9a7e33dc4781e272a9fa802a2ab9703fe4199d72ba1422c"
readonly KETO_IMAGE="oryd/keto@sha256:bfdb8b9e283aedbd9e1d005c44eb1bbefd2096b2cb5b5aa0b7360b6e39c178d2"
readonly RUN_ID="${FRF_AUTHORITY_RUN_ID:-c005-$RANDOM-$$}"
readonly RECEIPT_DIR="${FRF_AUTHORITY_RECEIPT_DIR:-$ROOT_DIR/target/integration-receipts}"
readonly RECEIPT_PATH="$RECEIPT_DIR/$RUN_ID.json"
readonly TENANT="00000000-0000-0000-0000-000000000051"
readonly CROSS_TENANT="00000000-0000-0000-0000-000000000052"
readonly CHANNEL="00000000-0000-0000-0000-0000c0050001"
readonly ALLOWED_EVENT="00000000-0000-0000-0000-0000c0050002"
readonly POSTGRES_NAME="frf-pri-$RUN_ID-postgres"
readonly KETO_NAME="frf-pri-$RUN_ID-keto"
readonly FABRIC_BUILD_DIR="${FRF_FABRIC_BUILD_DIR:-/tmp/frf-c005-fabric-build}"
readonly FABRIC_TARGET_DIR="${FRF_FABRIC_TARGET_DIR:-/tmp/frf-c005-fabric-target}"

RUN_DIR=""
GATE_PID=""
FIXTURE_PID=""
EXIT_STATUS=1
SCENARIOS=0
STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

fail() {
  echo "authority lifetime: $*" >&2
  return 1
}

free_port() {
  python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

sha256_file() {
  if [[ -f "$1" ]]; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    printf 'unavailable'
  fi
}

fabric_diff_hash() {
  git -C "$ROOT_DIR" diff --binary HEAD -- \
    .env.example \
    Cargo.lock \
    Cargo.toml \
    crates/frf-app/src/subscribe.rs \
    crates/frf-app/tests/subscribe_pipeline.rs \
    crates/frf-authz-keto \
    crates/frf-identity-ory | shasum -a 256 | awk '{print $1}'
}

fabric_status_hash() {
  git -C "$ROOT_DIR" status --short -- \
    .env.example \
    Cargo.lock \
    Cargo.toml \
    crates/frf-app/src/subscribe.rs \
    crates/frf-app/tests/subscribe_pipeline.rs \
    crates/frf-authz-keto \
    crates/frf-identity-ory \
    crates/frf-gateway/tests/authority_lifetime.rs \
    scripts/mint-authority-fixture.mjs \
    scripts/run-authority-lifetime.sh | shasum -a 256 | awk '{print $1}'
}

wait_http() {
  local url="$1"
  local expected="${2:-}"
  local deadline=$((SECONDS + 60))
  until body="$(curl -fsS "$url" 2>/dev/null)" && { [[ -z "$expected" ]] || grep -q "$expected" <<<"$body"; }; do
    ((SECONDS < deadline)) || fail "timed out waiting for $url"
    sleep 1
  done
}

write_receipt() {
  [[ -n "$RUN_DIR" && -d "$RUN_DIR" ]] || return
  mkdir -p "$RECEIPT_DIR"
  RECEIPT_PATH_VALUE="$RECEIPT_PATH" \
  RECEIPT_RUN_ID="$RUN_ID" \
  RECEIPT_STARTED="$STARTED_AT" \
  RECEIPT_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  RECEIPT_EXIT="$EXIT_STATUS" \
  RECEIPT_SCENARIOS="$SCENARIOS" \
  RECEIPT_FABRIC_REV="$(git -C "$ROOT_DIR" rev-parse HEAD)" \
  RECEIPT_FABRIC_DIFF="$(fabric_diff_hash)" \
  RECEIPT_FABRIC_STATUS="$(fabric_status_hash)" \
  RECEIPT_GATE_REV="$GATE_REVISION" \
  RECEIPT_POSTGRES="$POSTGRES_IMAGE" \
  RECEIPT_KETO="$KETO_IMAGE" \
  RECEIPT_TEST_HASH="$(sha256_file "$ROOT_DIR/crates/frf-gateway/tests/authority_lifetime.rs")" \
  RECEIPT_RUNNER_HASH="$(sha256_file "$ROOT_DIR/scripts/run-authority-lifetime.sh")" \
  RECEIPT_MINT_HASH="$(sha256_file "$ROOT_DIR/scripts/mint-authority-fixture.mjs")" \
  RECEIPT_GATE_LOG="$(sha256_file "$RUN_DIR/gate.log")" \
  RECEIPT_TEST_BUILD_LOG="$(sha256_file "$RUN_DIR/test-build.log")" \
  RECEIPT_TEST_LOG="$(sha256_file "$RUN_DIR/test.log")" \
  node <<'NODE'
const fs = require('node:fs');
const e = process.env;
const receipt = {
  schemaVersion: 1,
  runId: e.RECEIPT_RUN_ID,
  startedAt: e.RECEIPT_STARTED,
  finishedAt: e.RECEIPT_FINISHED,
  exitStatus: Number(e.RECEIPT_EXIT),
  scenariosCompleted: Number(e.RECEIPT_SCENARIOS),
  source: {
    fabric: e.RECEIPT_FABRIC_REV,
    fabricTrackedDiffSha256: e.RECEIPT_FABRIC_DIFF,
    fabricScopedStatusSha256: e.RECEIPT_FABRIC_STATUS,
    gate: e.RECEIPT_GATE_REV,
  },
  images: { postgres: e.RECEIPT_POSTGRES, keto: e.RECEIPT_KETO },
  sha256: {
    test: e.RECEIPT_TEST_HASH,
    runner: e.RECEIPT_RUNNER_HASH,
    mintFixture: e.RECEIPT_MINT_HASH,
    gateLog: e.RECEIPT_GATE_LOG,
    testBuildLog: e.RECEIPT_TEST_BUILD_LOG,
    testLog: e.RECEIPT_TEST_LOG,
  },
  credentials: 'ephemeral RSA keys and tokens removed during cleanup',
  cleanup: 'owned processes, containers, secrets and detached Gate worktree removed',
};
fs.writeFileSync(e.RECEIPT_PATH_VALUE, `${JSON.stringify(receipt, null, 2)}\n`, {mode: 0o600});
NODE
}

cleanup() {
  local status=$?
  trap - EXIT
  [[ "$EXIT_STATUS" == "0" ]] || EXIT_STATUS="$status"
  [[ -z "$GATE_PID" ]] || kill "$GATE_PID" 2>/dev/null || true
  [[ -z "$FIXTURE_PID" ]] || kill "$FIXTURE_PID" 2>/dev/null || true
  docker rm -f "$KETO_NAME" "$POSTGRES_NAME" >/dev/null 2>&1 || true
  if [[ "$EXIT_STATUS" != "0" && -n "$RUN_DIR" ]]; then
    for log in test-build.log gate-init.log gate.log test.log; do
      if [[ -f "$RUN_DIR/$log" ]]; then
        echo "--- $log (tail) ---" >&2
        tail -n 80 "$RUN_DIR/$log" >&2
      fi
    done
  fi
  if [[ -n "$RUN_DIR" && -d "$RUN_DIR/gate" ]]; then
    git -C "$GATE_REPO" worktree remove --force "$RUN_DIR/gate" >/dev/null 2>&1 || true
  fi
  write_receipt
  [[ -z "$RUN_DIR" ]] || rm -rf "$RUN_DIR"
  exit "$EXIT_STATUS"
}

seed_tuple() {
  local write_url="$1" subject="$2" relation="$3" object="$4"
  curl -fsS -X PUT "$write_url/admin/relation-tuples" \
    -H 'content-type: application/json' \
    --data "{\"namespace\":\"default\",\"object\":\"$object\",\"relation\":\"$relation\",\"subject_id\":\"$subject\"}" \
    >/dev/null
}

main() {
  for command_name in cargo curl docker git grep node openssl python3 shasum; do
    command -v "$command_name" >/dev/null || fail "missing command: $command_name"
  done
  [[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,42}$ ]] || fail "invalid run id"
  git -C "$GATE_REPO" cat-file -e "$GATE_REVISION^{commit}"
  RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/frf-pri-c005.XXXXXX")"
  trap cleanup EXIT
  git -C "$GATE_REPO" worktree add --detach "$RUN_DIR/gate" "$GATE_REVISION" >/dev/null

  CARGO_BUILD_BUILD_DIR="$FABRIC_BUILD_DIR" \
  CARGO_TARGET_DIR="$FABRIC_TARGET_DIR" \
  CARGO_BUILD_RUSTC_WRAPPER='' \
  RUSTC_WRAPPER='' \
    cargo test --locked -p frf-gateway --test authority_lifetime \
      --no-run >"$RUN_DIR/test-build.log" 2>&1

  local fixture_port gate_port admin_port postgres_port keto_read_port keto_write_port
  local postgres_password
  postgres_password="$(openssl rand -hex 24)"
  fixture_port="$(free_port)"
  gate_port="$(free_port)"
  admin_port="$(free_port)"
  node "$ROOT_DIR/scripts/mint-authority-fixture.mjs" \
    --out-dir "$RUN_DIR/keys" --tenant "$TENANT" --cross-tenant "$CROSS_TENANT" \
    --issuer "http://127.0.0.1:$fixture_port" --audience flint-gate-entry

  cat >"$RUN_DIR/fixture.py" <<'PY'
import http.server, pathlib, sys
root = pathlib.Path(sys.argv[1])
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/idp-jwks.json":
            body = (root / "idp-jwks.json").read_bytes()
            content_type = "application/json"
        else:
            body = self.headers.get("Authorization", "").encode()
            content_type = "text/plain"
        self.send_response(200)
        self.send_header("content-type", content_type)
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *_args):
        pass
http.server.ThreadingHTTPServer(("127.0.0.1", int(sys.argv[2])), Handler).serve_forever()
PY
  python3 "$RUN_DIR/fixture.py" "$RUN_DIR/keys" "$fixture_port" >"$RUN_DIR/fixture.log" 2>&1 &
  FIXTURE_PID=$!
  wait_http "http://127.0.0.1:$fixture_port/idp-jwks.json" 'pri-c005-idp'

  docker run -d --name "$POSTGRES_NAME" --label "frf.pri.run=$RUN_ID" \
    -e POSTGRES_USER=flint -e "POSTGRES_PASSWORD=$postgres_password" \
    -e POSTGRES_DB=flint_gate -p 127.0.0.1::5432 "$POSTGRES_IMAGE" >/dev/null
  postgres_port="$(docker port "$POSTGRES_NAME" 5432/tcp | awk -F: '{print $NF}')"
  local deadline=$((SECONDS + 60))
  until docker exec "$POSTGRES_NAME" pg_isready -U flint -d flint_gate >/dev/null 2>&1; do
    ((SECONDS < deadline)) || fail "Postgres did not become ready"
    sleep 1
  done

  docker run -d --name "$KETO_NAME" --label "frf.pri.run=$RUN_ID" \
    -v "$ROOT_DIR/deploy/keto:/etc/config/keto:ro" \
    -p 127.0.0.1::4466 -p 127.0.0.1::4467 "$KETO_IMAGE" \
    serve -c /etc/config/keto/keto.yml >/dev/null
  keto_read_port="$(docker port "$KETO_NAME" 4466/tcp | awk -F: '{print $NF}')"
  keto_write_port="$(docker port "$KETO_NAME" 4467/tcp | awk -F: '{print $NF}')"
  wait_http "http://127.0.0.1:$keto_read_port/health/ready"

  cat >"$RUN_DIR/gate.yaml" <<EOF
server:
  listen: "127.0.0.1:$gate_port"
  admin_listen: "127.0.0.1:$admin_port"
  allow_insecure_upstream: true
  strict_agent_governance: false
  tls:
    enabled: false
database:
  url: "postgres://flint:$postgres_password@127.0.0.1:$postgres_port/flint_gate"
  max_connections: 5
auth_providers:
  fixture_identity:
    type: jwt
    jwks_url: "http://127.0.0.1:$fixture_port/idp-jwks.json"
    issuer: "http://127.0.0.1:$fixture_port"
    audience: "flint-gate-entry"
    leeway_seconds: 0
jwt:
  signing_algorithm: "RS256"
  signing_key_path: "$RUN_DIR/keys/gate-private.pem"
  signing_key_id: "pri-c005-gate"
  issuer: "http://127.0.0.1:$gate_port"
  default_ttl_seconds: 120
token_exchange:
  enabled: false
authority:
  enabled: false
sites:
  - id: "authority-fixture"
    domains: ["*"]
    default_auth: fixture_identity
    default_upstream: "http://127.0.0.1:$fixture_port"
routes:
  - id: "mint"
    site: "authority-fixture"
    match:
      path: "/mint"
      methods: ["GET"]
    hooks:
      pre_request:
        - type: claims_enhancement
          config:
            mint_jwt:
              enabled: true
              additional_claims:
                aud: "frf-gateway"
                tenant_id: "{{ identity.metadata_public.tenant_id }}"
EOF

  local gate_target="${FRF_GATE_TARGET_DIR:-$ROOT_DIR/target/c005-gate}"
  CARGO_BUILD_BUILD_DIR="$gate_target" CARGO_TARGET_DIR="$gate_target" \
    CARGO_BUILD_RUSTC_WRAPPER='' RUSTC_WRAPPER='' cargo run --locked \
    --manifest-path "$RUN_DIR/gate/Cargo.toml" -p flint-gate -- \
    --config "$RUN_DIR/gate.yaml" --database-init-only \
    --jwt-public-key-path "$RUN_DIR/keys/gate-public.pem" >"$RUN_DIR/gate-init.log" 2>&1
  RUST_LOG=info FLINT_GATE_REQUIRE_DATABASE=true "$gate_target/debug/flint-gate" \
    --config "$RUN_DIR/gate.yaml" >"$RUN_DIR/gate.log" 2>&1 &
  GATE_PID=$!
  wait_http "http://127.0.0.1:$gate_port/.well-known/jwks.json" 'pri-c005-gate'

  local allowed_token denied_token cross_token
  allowed_token="$(curl -fsS -H "Authorization: Bearer $(<"$RUN_DIR/keys/allowed-inbound.txt")" "http://127.0.0.1:$gate_port/mint" | sed 's/^Bearer //')"
  denied_token="$(curl -fsS -H "Authorization: Bearer $(<"$RUN_DIR/keys/denied-inbound.txt")" "http://127.0.0.1:$gate_port/mint" | sed 's/^Bearer //')"
  cross_token="$(curl -fsS -H "Authorization: Bearer $(<"$RUN_DIR/keys/cross-inbound.txt")" "http://127.0.0.1:$gate_port/mint" | sed 's/^Bearer //')"
  [[ "$allowed_token" == *.*.* && "$denied_token" == *.*.* && "$cross_token" == *.*.* ]] \
    || fail "Gate did not mint all three JWTs"

  local keto_write="http://127.0.0.1:$keto_write_port"
  seed_tuple "$keto_write" user:allowed subscribe "$CHANNEL"
  seed_tuple "$keto_write" user:allowed view "$ALLOWED_EVENT"
  seed_tuple "$keto_write" user:cross subscribe "$CHANNEL"

  FRF_AUTHORITY_ISSUER="http://127.0.0.1:$gate_port" \
  FRF_AUTHORITY_AUDIENCE=frf-gateway \
  FRF_AUTHORITY_TENANT="$TENANT" \
  FRF_AUTHORITY_CROSS_TENANT="$CROSS_TENANT" \
  FRF_AUTHORITY_ALLOWED_TOKEN="$allowed_token" \
  FRF_AUTHORITY_DENIED_TOKEN="$denied_token" \
  FRF_AUTHORITY_CROSS_TOKEN="$cross_token" \
  FRF_AUTHORITY_JWKS_URL="http://127.0.0.1:$gate_port/.well-known/jwks.json" \
  FRF_AUTHORITY_KETO_READ_URL="http://127.0.0.1:$keto_read_port" \
  FRF_AUTHORITY_KETO_WRITE_URL="http://127.0.0.1:$keto_write_port" \
  CARGO_BUILD_BUILD_DIR="$FABRIC_BUILD_DIR" \
  CARGO_TARGET_DIR="$FABRIC_TARGET_DIR" \
  CARGO_BUILD_RUSTC_WRAPPER='' \
  RUSTC_WRAPPER='' \
    cargo test --locked -p frf-gateway --test authority_lifetime \
      real_gate_tokens_and_keto_enforce_subject_tenant_and_object \
      -- --ignored --exact --nocapture >"$RUN_DIR/test.log" 2>&1
  grep -q 'test result: ok. 1 passed; 0 failed; 0 ignored' "$RUN_DIR/test.log" \
    || fail "required authority test did not execute exactly once"
  SCENARIOS=7
  EXIT_STATUS=0
  echo "authority lifetime passed: 7 required identity/tenant/object/revocation scenarios"
}

main "$@"
