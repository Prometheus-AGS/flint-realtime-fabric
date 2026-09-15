#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT_DIR
readonly COMPOSE_FILE="$ROOT_DIR/compose.integration.yml"
readonly REQUIRED_SCENARIOS="${FRF_REQUIRED_SCENARIOS:-2}"
readonly TEST_FILTER="${FRF_TEST_FILTER:-subscriber_receives_published_event}"
readonly RUN_ID="${FRF_INTEGRATION_RUN_ID:-$(date -u +%Y%m%d%H%M%S)-$$}"
readonly PROJECT="frf-pri-$RUN_ID"
readonly RECEIPT_DIR="${FRF_INTEGRATION_RECEIPT_DIR:-$ROOT_DIR/target/integration-receipts}"
readonly RECEIPT_PATH="$RECEIPT_DIR/$RUN_ID.json"

RUN_DIR=""
IGGY_PORT=""
STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
COMPOSE_STARTED=0
NEGATIVE_STATUS="not-run"
POSITIVE_STATUS="not-run"
SCENARIOS_COMPLETED=0
CLEANUP_STATUS="not-run"
IMAGE_ID="unavailable"
IMAGE_DIGEST="unavailable"
HASH_COMMAND=""

fail() {
  echo "local integration: $*" >&2
  return 1
}

require_prerequisites() {
  local command_name
  for command_name in awk bash cargo date dirname docker git grep mkdir mktemp node rm sleep tail; do
    command -v "$command_name" >/dev/null 2>&1 || fail "missing required command: $command_name"
  done
  docker compose version >/dev/null 2>&1 || fail "docker compose plugin is unavailable"
  if command -v sha256sum >/dev/null 2>&1; then
    HASH_COMMAND="sha256sum"
  elif command -v shasum >/dev/null 2>&1; then
    HASH_COMMAND="shasum"
  else
    fail "missing required SHA-256 command: sha256sum or shasum"
  fi
}

sha256_file() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    echo "unavailable"
  elif [[ "$HASH_COMMAND" == "sha256sum" ]]; then
    sha256sum "$file" | awk '{print $1}'
  else
    shasum -a 256 "$file" | awk '{print $1}'
  fi
}

assert_owned_project() {
  [[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,42}$ ]] || fail "invalid run id: use 1-43 lowercase letters, digits, or hyphens"
  [[ "$PROJECT" =~ ^frf-pri-[a-z0-9][a-z0-9-]{0,42}$ ]] || fail "refusing unowned Compose project: $PROJECT"
}

assert_no_existing_resources() {
  local containers volumes
  containers="$(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT")"
  volumes="$(docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT")"
  [[ -z "$containers" && -z "$volumes" ]] || fail "Compose project already owns resources: $PROJECT"
}

wait_for_iggy() {
  local container_id="$1"
  local deadline=$((SECONDS + 30))
  until docker exec "$container_id" iggy \
    --transport tcp \
    --tcp-server-address 127.0.0.1:8090 \
    --username "$FRF_IGGY_USERNAME" \
    --password "$FRF_IGGY_PASSWORD" \
    me >/dev/null 2>&1; do
    ((SECONDS < deadline)) || fail "Iggy protocol login did not succeed before the deadline"
    sleep 1
  done
}

write_receipt() {
  local exit_status="$1"
  local negative_log="$RUN_DIR/negative.log"
  local positive_log="$RUN_DIR/positive.log"
  local compose_log="$RUN_DIR/compose.log"
  local commit
  commit="$(git -C "$ROOT_DIR" rev-parse HEAD 2>/dev/null || echo unavailable)"
  mkdir -p "$RECEIPT_DIR"
  RECEIPT_PATH_VALUE="$RECEIPT_PATH" \
  RECEIPT_RUN_ID="$RUN_ID" \
  RECEIPT_PROJECT="$PROJECT" \
  RECEIPT_STARTED="$STARTED_AT" \
  RECEIPT_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  RECEIPT_EXIT="$exit_status" \
  RECEIPT_COMMIT="$commit" \
  RECEIPT_PORT="$IGGY_PORT" \
  RECEIPT_IMAGE_ID="$IMAGE_ID" \
  RECEIPT_IMAGE_DIGEST="$IMAGE_DIGEST" \
  RECEIPT_REQUIRED="$REQUIRED_SCENARIOS" \
  RECEIPT_COMPLETED="$SCENARIOS_COMPLETED" \
  RECEIPT_NEGATIVE="$NEGATIVE_STATUS" \
  RECEIPT_POSITIVE="$POSITIVE_STATUS" \
  RECEIPT_CLEANUP="$CLEANUP_STATUS" \
  RECEIPT_COMPOSE_HASH="$(sha256_file "$COMPOSE_FILE")" \
  RECEIPT_TEST_HASH="$(sha256_file "$ROOT_DIR/crates/frf-gateway/tests/subscribe_mux.rs")" \
  RECEIPT_RUNNER_HASH="$(sha256_file "$ROOT_DIR/scripts/run-local-integration.sh")" \
  RECEIPT_NEGATIVE_HASH="$(sha256_file "$negative_log")" \
  RECEIPT_POSITIVE_HASH="$(sha256_file "$positive_log")" \
  RECEIPT_COMPOSE_LOG_HASH="$(sha256_file "$compose_log")" \
  node <<'NODE'
const fs = require('node:fs');
const env = process.env;
const receipt = {
  schemaVersion: 1,
  runId: env.RECEIPT_RUN_ID,
  sourceCommit: env.RECEIPT_COMMIT,
  startedAt: env.RECEIPT_STARTED,
  finishedAt: env.RECEIPT_FINISHED,
  exitStatus: Number(env.RECEIPT_EXIT),
  topology: {
    composeProject: env.RECEIPT_PROJECT,
    composeFile: 'compose.integration.yml',
    iggyEndpoint: `127.0.0.1:${env.RECEIPT_PORT}`,
    imageId: env.RECEIPT_IMAGE_ID,
    imageDigest: env.RECEIPT_IMAGE_DIGEST,
  },
  scenarios: {
    required: Number(env.RECEIPT_REQUIRED),
    completed: Number(env.RECEIPT_COMPLETED),
    disabledDeliveryExit: env.RECEIPT_NEGATIVE,
    restoredDeliveryExit: env.RECEIPT_POSITIVE,
  },
  cleanup: {
    status: env.RECEIPT_CLEANUP,
    scope: `containers and volumes labeled com.docker.compose.project=${env.RECEIPT_PROJECT}`,
  },
  sha256: {
    compose: env.RECEIPT_COMPOSE_HASH,
    test: env.RECEIPT_TEST_HASH,
    runner: env.RECEIPT_RUNNER_HASH,
    disabledDeliveryLog: env.RECEIPT_NEGATIVE_HASH,
    restoredDeliveryLog: env.RECEIPT_POSITIVE_HASH,
    composeLog: env.RECEIPT_COMPOSE_LOG_HASH,
  },
  credentials: 'disposable Iggy credentials, RS256 keypair and JWT removed after cleanup',
};
fs.writeFileSync(env.RECEIPT_PATH_VALUE, `${JSON.stringify(receipt, null, 2)}\n`, {mode: 0o600});
NODE
  echo "integration receipt: $RECEIPT_PATH"
}

cleanup() {
  local exit_status=$?
  trap - EXIT
  set +e
  if [[ "$COMPOSE_STARTED" == "1" ]]; then
    if assert_owned_project && docker compose \
      -p "$PROJECT" -f "$COMPOSE_FILE" down -v --remove-orphans >/dev/null 2>&1; then
      local containers volumes
      containers="$(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT")"
      volumes="$(docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT")"
      if [[ -z "$containers" && -z "$volumes" ]]; then
        CLEANUP_STATUS="owned-resources-removed"
      else
        CLEANUP_STATUS="owned-resources-remain"
        exit_status=1
      fi
    else
      CLEANUP_STATUS="compose-down-failed"
      exit_status=1
    fi
  fi
  if [[ -n "$RUN_DIR" && -d "$RUN_DIR" ]]; then
    write_receipt "$exit_status"
    rm -rf "$RUN_DIR"
  fi
  exit "$exit_status"
}

run_required_test() {
  local mode="$1"
  local log="$2"
  local status
  local -a command=(
    cargo test -p frf-gateway --test subscribe_mux "$TEST_FILTER"
    -- --ignored --exact --nocapture
  )
  set +e
  if [[ "$mode" == "disabled" ]]; then
    FRF_DISABLE_DELIVERY=1 "${command[@]}" >"$log" 2>&1
  else
    (unset FRF_DISABLE_DELIVERY; "${command[@]}") >"$log" 2>&1
  fi
  status=$?
  set -e
  echo "$status"
}

main() {
  require_prerequisites
  assert_owned_project
  [[ "$REQUIRED_SCENARIOS" =~ ^[1-9][0-9]*$ ]] || fail "required scenario count must be a positive integer"
  [[ "$REQUIRED_SCENARIOS" == "2" ]] || fail "this fixture defines exactly 2 required scenarios"
  [[ "$TEST_FILTER" == "subscriber_receives_published_event" ]] || fail "required test filter cannot be changed or skipped"
  [[ -f "$COMPOSE_FILE" ]] || fail "missing required fixture: $COMPOSE_FILE"
  if [[ "${1:-}" == "--preflight-only" ]]; then
    echo "local integration preflight passed"
    return
  fi
  [[ $# == 0 ]] || fail "unknown argument: $1"

  assert_no_existing_resources
  RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/frf-pri-c002.XXXXXX")"
  trap cleanup EXIT
  mkdir -p "$RECEIPT_DIR"

  node "$ROOT_DIR/scripts/mint-e2e-jwt.mjs" \
    --out-dir "$RUN_DIR/credentials" \
    --sub user:pri-c002 \
    --aud frf-gateway \
    --iss frf-e2e \
    --tenant 00000000-0000-0000-0000-000000000001 \
    --ttl 600 >"$RUN_DIR/token.stdout"
  export FRF_TEST_JWT
  FRF_TEST_JWT="$(<"$RUN_DIR/credentials/jwt.txt")"
  export FRF_TEST_JWKS_FILE="$RUN_DIR/credentials/jwks.json"
  local iggy_credentials
  iggy_credentials="$(node -e "const {randomBytes}=require('node:crypto'); console.log('pri'+randomBytes(8).toString('hex')+':'+randomBytes(24).toString('hex'))")"
  export FRF_IGGY_USERNAME="${iggy_credentials%%:*}"
  export FRF_IGGY_PASSWORD="${iggy_credentials#*:}"
  docker compose -p "$PROJECT" -f "$COMPOSE_FILE" config -q
  COMPOSE_STARTED=1
  docker compose -p "$PROJECT" -f "$COMPOSE_FILE" up -d \
    >"$RUN_DIR/compose.log" 2>&1
  local container_id port_mapping repo_digests
  port_mapping="$(docker compose -p "$PROJECT" -f "$COMPOSE_FILE" port iggy 8090)"
  IGGY_PORT="${port_mapping##*:}"
  [[ "$IGGY_PORT" =~ ^[1-9][0-9]*$ ]] || fail "Docker returned an invalid Iggy port mapping: $port_mapping"
  export FRF_TEST_IGGY_URL="iggy://$FRF_IGGY_USERNAME:$FRF_IGGY_PASSWORD@127.0.0.1:$IGGY_PORT"
  container_id="$(docker compose -p "$PROJECT" -f "$COMPOSE_FILE" ps -q iggy)"
  wait_for_iggy "$container_id"
  IMAGE_ID="$(docker inspect "$container_id" --format '{{.Image}}')"
  repo_digests="$(docker image inspect "$IMAGE_ID" --format '{{join .RepoDigests ","}}' 2>/dev/null || true)"
  [[ -n "$repo_digests" ]] && IMAGE_DIGEST="$repo_digests"

  NEGATIVE_STATUS="$(run_required_test disabled "$RUN_DIR/negative.log")"
  [[ "$NEGATIVE_STATUS" != "0" ]] || fail "disabled delivery did not fail the required assertion"
  grep -q "required delivery missing" "$RUN_DIR/negative.log" || fail "disabled-delivery failure was not the required assertion"
  SCENARIOS_COMPLETED=$((SCENARIOS_COMPLETED + 1))

  POSITIVE_STATUS="$(run_required_test enabled "$RUN_DIR/positive.log")"
  [[ "$POSITIVE_STATUS" == "0" ]] || {
    tail -80 "$RUN_DIR/positive.log" >&2
    fail "restored delivery scenario failed"
  }
  grep -Eq "test result: ok\. 1 passed; 0 failed; 0 ignored" "$RUN_DIR/positive.log" || fail "required test was skipped or did not run exactly once"
  grep -q "required delivery received:" "$RUN_DIR/positive.log" || fail "delivery assertion marker is absent"
  SCENARIOS_COMPLETED=$((SCENARIOS_COMPLETED + 1))
  [[ "$SCENARIOS_COMPLETED" == "$REQUIRED_SCENARIOS" ]] || fail "required scenario count mismatch"

  echo "local integration passed: $SCENARIOS_COMPLETED required scenarios"
}

main "$@"
