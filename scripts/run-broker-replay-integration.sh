#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT_DIR
readonly BASE_COMPOSE="$ROOT_DIR/compose.integration.yml"
readonly REPLAY_COMPOSE="$ROOT_DIR/compose.broker-replay.yml"
readonly PINNED_IGGY_IMAGE="iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5"
readonly RUN_ID="${FRF_BROKER_REPLAY_RUN_ID:-$(date -u +%Y%m%d%H%M%S)-$$}"
readonly PROJECT="frf-broker-replay-$RUN_ID"
readonly OUTPUT_ROOT="${FRF_BROKER_REPLAY_OUTPUT_DIR:-$ROOT_DIR/target/broker-replay-receipts}"
readonly OUTPUT_DIR="$OUTPUT_ROOT/$RUN_ID"
readonly RECEIPT="$OUTPUT_DIR/receipt.json"

STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
COMPOSE_STARTED=0
CLEANUP_STATUS="not-run"
CORE_STATUS="not-run"
SEED_STATUS="not-run"
VERIFY_STATUS="not-run"
RESTART_STATUS="not-run"
IGGY_PORT=""
IMAGE_ID="unavailable"
IMAGE_DIGEST="unavailable"
HASH_COMMAND=""

fail() {
  echo "broker replay integration: $*" >&2
  return 1
}

require_prerequisites() {
  local command_name
  for command_name in awk bash cargo date docker git grep mkdir nc node rm sleep; do
    command -v "$command_name" >/dev/null 2>&1 || fail "missing command: $command_name"
  done
  docker compose version >/dev/null 2>&1 || fail "docker compose plugin is unavailable"
  if command -v sha256sum >/dev/null 2>&1; then
    HASH_COMMAND="sha256sum"
  elif command -v shasum >/dev/null 2>&1; then
    HASH_COMMAND="shasum"
  else
    fail "missing SHA-256 command"
  fi
}

sha256_file() {
  local path="$1"
  if [[ ! -f "$path" && "$path" != "/dev/stdin" ]]; then
    echo "unavailable"
    return
  fi
  if [[ "$HASH_COMMAND" == "sha256sum" ]]; then
    sha256sum "$path" | awk '{print $1}'
  else
    shasum -a 256 "$path" | awk '{print $1}'
  fi
}

assert_owned_project() {
  [[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,42}$ ]] || fail "invalid run id"
  [[ "$PROJECT" =~ ^frf-broker-replay-[a-z0-9][a-z0-9-]{0,42}$ ]] ||
    fail "refusing unowned Compose project: $PROJECT"
}

compose() {
  docker compose -p "$PROJECT" -f "$BASE_COMPOSE" -f "$REPLAY_COMPOSE" "$@"
}

wait_for_iggy() {
  local container_id="$1"
  local deadline=$((SECONDS + 45))
  until docker exec "$container_id" iggy \
    --transport tcp \
    --tcp-server-address 127.0.0.1:8090 \
    --username "$FRF_IGGY_USERNAME" \
    --password "$FRF_IGGY_PASSWORD" \
    me >/dev/null 2>&1; do
    ((SECONDS < deadline)) || fail "Iggy protocol login timed out"
    sleep 1
  done
}

run_test() {
  local name="$1"
  local log="$2"
  CARGO_BUILD_BUILD_DIR='/tmp/frf-c007-build/{workspace-path-hash}' \
    RUSTC_WRAPPER='' \
    cargo test --offline -p frf-broker-iggy --test replay_contract "$name" --locked -- \
      --ignored --exact --nocapture >"$log" 2>&1
  grep -q "test result: ok. 1 passed; 0 failed; 0 ignored" "$log" ||
    fail "$name was skipped or did not run exactly once"
}

write_receipt() {
  local exit_status="$1"
  local source_commit artifact_set_hash binding_file relative_path
  source_commit="$(git -C "$ROOT_DIR" rev-parse HEAD)"
  mkdir -p "$OUTPUT_DIR"
  binding_file="$OUTPUT_DIR/source-binding.sha256"
  : >"$binding_file"
  for relative_path in \
    crates/frf-broker-iggy/src/broker.rs \
    crates/frf-broker-iggy/src/broker_tests.rs \
    crates/frf-broker-iggy/src/lib.rs \
    crates/frf-broker-iggy/src/position.rs \
    crates/frf-broker-iggy/tests/replay_contract.rs \
    crates/frf-ports/src/log_broker.rs \
    compose.broker-replay.yml \
    scripts/run-broker-replay-integration.sh; do
    printf '%s  %s\n' "$(sha256_file "$ROOT_DIR/$relative_path")" "$relative_path" >>"$binding_file"
  done
  artifact_set_hash="$(sha256_file "$binding_file")"
  RECEIPT_PATH="$RECEIPT" \
  RECEIPT_RUN_ID="$RUN_ID" \
  RECEIPT_PROJECT="$PROJECT" \
  RECEIPT_STARTED="$STARTED_AT" \
  RECEIPT_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  RECEIPT_EXIT="$exit_status" \
  RECEIPT_SOURCE_COMMIT="$source_commit" \
  RECEIPT_ARTIFACT_SET_HASH="$artifact_set_hash" \
  RECEIPT_PORT="$IGGY_PORT" \
  RECEIPT_IMAGE_ID="$IMAGE_ID" \
  RECEIPT_IMAGE_DIGEST="$IMAGE_DIGEST" \
  RECEIPT_CORE="$CORE_STATUS" \
  RECEIPT_SEED="$SEED_STATUS" \
  RECEIPT_RESTART="$RESTART_STATUS" \
  RECEIPT_VERIFY="$VERIFY_STATUS" \
  RECEIPT_CLEANUP="$CLEANUP_STATUS" \
  RECEIPT_BROKER_HASH="$(sha256_file "$ROOT_DIR/crates/frf-broker-iggy/src/broker.rs")" \
  RECEIPT_BROKER_TESTS_HASH="$(sha256_file "$ROOT_DIR/crates/frf-broker-iggy/src/broker_tests.rs")" \
  RECEIPT_BROKER_LIB_HASH="$(sha256_file "$ROOT_DIR/crates/frf-broker-iggy/src/lib.rs")" \
  RECEIPT_POSITION_HASH="$(sha256_file "$ROOT_DIR/crates/frf-broker-iggy/src/position.rs")" \
  RECEIPT_TEST_HASH="$(sha256_file "$ROOT_DIR/crates/frf-broker-iggy/tests/replay_contract.rs")" \
  RECEIPT_PORT_HASH="$(sha256_file "$ROOT_DIR/crates/frf-ports/src/log_broker.rs")" \
  RECEIPT_COMPOSE_HASH="$(sha256_file "$REPLAY_COMPOSE")" \
  RECEIPT_RUNNER_HASH="$(sha256_file "$ROOT_DIR/scripts/run-broker-replay-integration.sh")" \
  RECEIPT_CORE_LOG_HASH="$(sha256_file "$OUTPUT_DIR/core.log")" \
  RECEIPT_SEED_LOG_HASH="$(sha256_file "$OUTPUT_DIR/seed.log")" \
  RECEIPT_VERIFY_LOG_HASH="$(sha256_file "$OUTPUT_DIR/verify.log")" \
  RECEIPT_SERVER_LOG_HASH="$(sha256_file "$OUTPUT_DIR/server.log")" \
  node <<'NODE'
const fs = require('node:fs');
const e = process.env;
const receipt = {
  schemaVersion: 1,
  runId: e.RECEIPT_RUN_ID,
  source: {
    commit: e.RECEIPT_SOURCE_COMMIT,
    candidateArtifactSetSha256: e.RECEIPT_ARTIFACT_SET_HASH,
  },
  startedAt: e.RECEIPT_STARTED,
  finishedAt: e.RECEIPT_FINISHED,
  exitStatus: Number(e.RECEIPT_EXIT),
  topology: {
    composeProject: e.RECEIPT_PROJECT,
    composeFiles: ['compose.integration.yml', 'compose.broker-replay.yml'],
    endpoint: `127.0.0.1:${e.RECEIPT_PORT}`,
    imageId: e.RECEIPT_IMAGE_ID,
    imageDigest: e.RECEIPT_IMAGE_DIGEST,
    partitions: 1,
  },
  durability: {
    messageSaver: {enabled: true, enforceFsync: true, interval: '1 s'},
    partition: {enforceFsync: true, messagesRequiredToSave: 1},
    state: {enforceFsync: true},
    messageDeduplication: {enabled: true, maxEntries: 10000, expiry: '24 h'},
    segmentSize: '1 KB',
    topicRetentionSeconds: 86400,
  },
  scenarios: {
    crashBeforeCheckpoint: e.RECEIPT_CORE,
    checkpointAndInclusiveResume: e.RECEIPT_CORE,
    independentConsumersAndCancellation: e.RECEIPT_CORE,
    producerRestartPosition: e.RECEIPT_CORE,
    concurrentPublishPosition: e.RECEIPT_CORE,
    seedBeforeServerRestart: e.RECEIPT_SEED,
    serverRestart: e.RECEIPT_RESTART,
    durableReplayAfterServerRestart: e.RECEIPT_VERIFY,
    stableIdentityDeduplicationAfterRestart: e.RECEIPT_VERIFY,
  },
  cleanup: {status: e.RECEIPT_CLEANUP, composeProject: e.RECEIPT_PROJECT},
  sha256: {
    broker: e.RECEIPT_BROKER_HASH,
    brokerUnitTests: e.RECEIPT_BROKER_TESTS_HASH,
    brokerLib: e.RECEIPT_BROKER_LIB_HASH,
    positionMapping: e.RECEIPT_POSITION_HASH,
    test: e.RECEIPT_TEST_HASH,
    port: e.RECEIPT_PORT_HASH,
    composeOverride: e.RECEIPT_COMPOSE_HASH,
    runner: e.RECEIPT_RUNNER_HASH,
    sourceBinding: e.RECEIPT_ARTIFACT_SET_HASH,
    coreLog: e.RECEIPT_CORE_LOG_HASH,
    seedLog: e.RECEIPT_SEED_LOG_HASH,
    verifyLog: e.RECEIPT_VERIFY_LOG_HASH,
    serverLog: e.RECEIPT_SERVER_LOG_HASH,
  },
  credentials: 'disposable Iggy credentials removed with the owned Compose project',
};
fs.writeFileSync(e.RECEIPT_PATH, `${JSON.stringify(receipt, null, 2)}\n`, {mode: 0o600});
NODE
  echo "broker replay receipt: $RECEIPT"
}

cleanup() {
  local exit_status=$?
  trap - EXIT
  set +e
  if [[ "$COMPOSE_STARTED" == "1" ]]; then
    compose logs --no-color >"$OUTPUT_DIR/server.log" 2>&1
    local container_id
    container_id="$(compose ps -q iggy)"
    if [[ -n "$container_id" ]]; then
      docker cp "$container_id:/data/logs" "$OUTPUT_DIR/iggy-logs" >/dev/null 2>&1 || true
    fi
    if assert_owned_project && compose down -v --remove-orphans >/dev/null 2>&1; then
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
  write_receipt "$exit_status"
  exit "$exit_status"
}

main() {
  require_prerequisites
  assert_owned_project
  [[ -f "$BASE_COMPOSE" && -f "$REPLAY_COMPOSE" ]] || fail "Compose fixture missing"
  export FRF_IGGY_IMAGE="${FRF_IGGY_IMAGE:-$PINNED_IGGY_IMAGE}"
  [[ "$FRF_IGGY_IMAGE" =~ @sha256:[0-9a-f]{64}$ ]] || fail "Iggy image must be immutable"
  [[ "$FRF_IGGY_IMAGE" == "$PINNED_IGGY_IMAGE" ]] || fail "Iggy image differs from the pinned-fork server digest"
  local credentials
  credentials="$(node -e "const {randomBytes}=require('node:crypto'); console.log('c007'+randomBytes(8).toString('hex')+':'+randomBytes(24).toString('hex'))")"
  export FRF_IGGY_USERNAME="${credentials%%:*}"
  export FRF_IGGY_PASSWORD="${credentials#*:}"
  if [[ "${1:-}" == "--preflight-only" ]]; then
    compose config -q
    echo "broker replay integration preflight passed"
    return
  fi
  [[ $# == 0 ]] || fail "unknown argument: $1"

  mkdir -p "$OUTPUT_DIR"
  compose config -q
  trap cleanup EXIT
  COMPOSE_STARTED=1
  compose up -d >"$OUTPUT_DIR/compose.log" 2>&1

  local container_id port_mapping repo_digests
  container_id="$(compose ps -q iggy)"
  port_mapping="$(compose port iggy 8090)"
  IGGY_PORT="${port_mapping##*:}"
  [[ "$IGGY_PORT" =~ ^[1-9][0-9]*$ ]] || fail "invalid Iggy port mapping"
  wait_for_iggy "$container_id"
  export FRF_BROKER_REPLAY_IGGY_URL="iggy://$FRF_IGGY_USERNAME:$FRF_IGGY_PASSWORD@127.0.0.1:$IGGY_PORT"
  IMAGE_ID="$(docker inspect "$container_id" --format '{{.Image}}')"
  repo_digests="$(docker image inspect "$IMAGE_ID" --format '{{join .RepoDigests ","}}')"
  IMAGE_DIGEST="$repo_digests"
  [[ ",$repo_digests," == *",$PINNED_IGGY_IMAGE,"* ]] || fail "running Iggy image digest mismatch"

  run_test broker_replay_contract "$OUTPUT_DIR/core.log"
  grep -q BROKER_REPLAY_CORE_PASS "$OUTPUT_DIR/core.log" || fail "core assertion marker missing"
  CORE_STATUS="pass"

  run_test seed_broker_restart_state "$OUTPUT_DIR/seed.log"
  grep -q BROKER_RESTART_SEED_PASS "$OUTPUT_DIR/seed.log" || fail "restart seed marker missing"
  SEED_STATUS="pass"

  compose restart iggy >>"$OUTPUT_DIR/compose.log" 2>&1
  container_id="$(compose ps -q iggy)"
  port_mapping="$(compose port iggy 8090)"
  IGGY_PORT="${port_mapping##*:}"
  [[ "$IGGY_PORT" =~ ^[1-9][0-9]*$ ]] || fail "invalid Iggy port mapping after restart"
  export FRF_BROKER_REPLAY_IGGY_URL="iggy://$FRF_IGGY_USERNAME:$FRF_IGGY_PASSWORD@127.0.0.1:$IGGY_PORT"
  wait_for_iggy "$container_id"
  local host_deadline=$((SECONDS + 30))
  until nc -z 127.0.0.1 "$IGGY_PORT"; do
    ((SECONDS < host_deadline)) || fail "Iggy host port timed out after restart"
    sleep 1
  done
  RESTART_STATUS="pass"

  run_test verify_broker_restart_state "$OUTPUT_DIR/verify.log"
  grep -q BROKER_RESTART_VERIFY_PASS "$OUTPUT_DIR/verify.log" || fail "restart verify marker missing"
  VERIFY_STATUS="pass"
  echo "broker replay integration passed"
}

main "$@"
