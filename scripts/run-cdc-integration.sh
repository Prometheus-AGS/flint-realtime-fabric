#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT_DIR
readonly BASE_COMPOSE="$ROOT_DIR/compose.integration.yml"
readonly CDC_COMPOSE="$ROOT_DIR/compose.cdc-integration.yml"
readonly PINNED_IGGY_IMAGE="iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5"
readonly PINNED_POSTGRES_IMAGE="postgres@sha256:18cfe3ef5e6815560c98237d6216d1e5119702fb0f3894c8785dd58b8bbe5d73"
readonly RUN_ID="${FRF_CDC_RUN_ID:-$(date -u +%Y%m%d%H%M%S)-$$}"
readonly PROJECT="frf-cdc-$RUN_ID"
readonly OUTPUT_ROOT="${FRF_CDC_OUTPUT_DIR:-$ROOT_DIR/target/cdc-integration-receipts}"
readonly OUTPUT_DIR="$OUTPUT_ROOT/$RUN_ID"
readonly RECEIPT="$OUTPUT_DIR/receipt.json"

STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
COMPOSE_STARTED=0
CLEANUP_STATUS="not-run"
TEST_STATUS="not-run"
IGGY_PORT=""
POSTGRES_PORT=""
IGGY_IMAGE_ID="unavailable"
POSTGRES_IMAGE_ID="unavailable"
HASH_COMMAND=""

fail() {
  echo "CDC integration: $*" >&2
  return 1
}

require_prerequisites() {
  local command_name
  for command_name in awk bash cargo date docker git grep mkdir node sleep; do
    command -v "$command_name" >/dev/null 2>&1 || fail "missing command: $command_name"
  done
  docker compose version >/dev/null 2>&1 || fail "Docker Compose is unavailable"
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
  if [[ ! -f "$path" ]]; then
    echo "unavailable"
  elif [[ "$HASH_COMMAND" == "sha256sum" ]]; then
    sha256sum "$path" | awk '{print $1}'
  else
    shasum -a 256 "$path" | awk '{print $1}'
  fi
}

assert_owned_project() {
  [[ "$RUN_ID" =~ ^[a-z0-9][a-z0-9-]{0,42}$ ]] || fail "invalid run id"
  [[ "$PROJECT" =~ ^frf-cdc-[a-z0-9][a-z0-9-]{0,42}$ ]] ||
    fail "refusing unowned Compose project: $PROJECT"
}

compose() {
  docker compose -p "$PROJECT" -f "$BASE_COMPOSE" -f "$CDC_COMPOSE" "$@"
}

wait_for_services() {
  local iggy_id="$1"
  local postgres_id="$2"
  local deadline=$((SECONDS + 60))
  until docker exec "$iggy_id" iggy \
    --transport tcp \
    --tcp-server-address 127.0.0.1:8090 \
    --username "$FRF_IGGY_USERNAME" \
    --password "$FRF_IGGY_PASSWORD" me >/dev/null 2>&1; do
    ((SECONDS < deadline)) || fail "Iggy protocol login timed out"
    sleep 1
  done
  until docker exec "$postgres_id" pg_isready \
    -U "$FRF_CDC_POSTGRES_USER" -d "$FRF_CDC_POSTGRES_DB" >/dev/null 2>&1; do
    ((SECONDS < deadline)) || fail "PostgreSQL readiness timed out"
    sleep 1
  done
}

write_receipt() {
  local exit_status="$1"
  local binding_file="$OUTPUT_DIR/source-binding.sha256"
  local path artifact_set_hash
  : >"$binding_file"
  for path in \
    .env.example .kbd-orchestrator/constraints.md CLAUDE.md \
    Cargo.lock Cargo.toml compose.yml compose.cdc-integration.yml \
    crates/frf-gateway/src/config/mod.rs \
    crates/frf-gateway/src/config/tests.rs \
    crates/frf-gateway/src/config/validation.rs \
    crates/frf-gateway/src/main.rs \
    crates/frf-gateway/Cargo.toml \
    crates/frf-postgres-cdc/Cargo.toml \
    crates/frf-postgres-cdc/src/canonical.rs \
    crates/frf-postgres-cdc/src/catalog.rs \
    crates/frf-postgres-cdc/src/catalog_validation.rs \
    crates/frf-postgres-cdc/src/config.rs \
    crates/frf-postgres-cdc/src/consumer.rs \
    crates/frf-postgres-cdc/src/decode.rs \
    crates/frf-postgres-cdc/src/lib.rs \
    crates/frf-postgres-cdc/src/model.rs \
    crates/frf-postgres-cdc/src/transaction.rs \
    crates/frf-postgres-cdc/src/transaction_tests.rs \
    crates/frf-gateway/tests/cdc_integration.rs \
    crates/frf-gateway/tests/support/cdc/mod.rs \
    deploy/postgres/init.sql docs/ENVIRONMENT.md docs/RUNBOOK.md \
    scripts/run-cdc-integration.sh; do
    printf '%s  %s\n' "$(sha256_file "$ROOT_DIR/$path")" "$path" >>"$binding_file"
  done
  artifact_set_hash="$(sha256_file "$binding_file")"
  RECEIPT_PATH="$RECEIPT" \
  RECEIPT_STARTED="$STARTED_AT" \
  RECEIPT_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  RECEIPT_EXIT="$exit_status" \
  RECEIPT_RUN_ID="$RUN_ID" \
  RECEIPT_PROJECT="$PROJECT" \
  RECEIPT_SOURCE_COMMIT="$(git -C "$ROOT_DIR" rev-parse HEAD)" \
  RECEIPT_BINDING="$artifact_set_hash" \
  RECEIPT_IGGY_PORT="$IGGY_PORT" \
  RECEIPT_POSTGRES_PORT="$POSTGRES_PORT" \
  RECEIPT_IGGY_ID="$IGGY_IMAGE_ID" \
  RECEIPT_POSTGRES_ID="$POSTGRES_IMAGE_ID" \
  RECEIPT_TEST_STATUS="$TEST_STATUS" \
  RECEIPT_CLEANUP="$CLEANUP_STATUS" \
  RECEIPT_TEST_LOG_HASH="$(sha256_file "$OUTPUT_DIR/test.log")" \
  RECEIPT_SERVER_LOG_HASH="$(sha256_file "$OUTPUT_DIR/server.log")" \
  node <<'NODE'
const fs = require('node:fs');
const e = process.env;
const receipt = {
  schemaVersion: 1,
  runId: e.RECEIPT_RUN_ID,
  source: {commit: e.RECEIPT_SOURCE_COMMIT, candidateArtifactSetSha256: e.RECEIPT_BINDING},
  startedAt: e.RECEIPT_STARTED,
  finishedAt: e.RECEIPT_FINISHED,
  exitStatus: Number(e.RECEIPT_EXIT),
  topology: {
    composeProject: e.RECEIPT_PROJECT,
    composeFiles: ['compose.integration.yml', 'compose.cdc-integration.yml'],
    iggyEndpoint: `127.0.0.1:${e.RECEIPT_IGGY_PORT}`,
    postgresEndpoint: `127.0.0.1:${e.RECEIPT_POSTGRES_PORT}`,
    iggyImage: {reference: process.env.FRF_IGGY_IMAGE, id: e.RECEIPT_IGGY_ID},
    postgresImage: {reference: process.env.FRF_POSTGRES_IMAGE, id: e.RECEIPT_POSTGRES_ID},
  },
  scenarios: {
    committedInsertUpdateDelete: e.RECEIPT_TEST_STATUS,
    rollbackAndMultiRowAtomicity: e.RECEIPT_TEST_STATUS,
    compositeNonFirstNonUuidKey: e.RECEIPT_TEST_STATUS,
    columnAndFixedTenant: e.RECEIPT_TEST_STATUS,
    typedValuesAndUnchangedToast: e.RECEIPT_TEST_STATUS,
    unsupportedMappingAndReplicaIdentity: e.RECEIPT_TEST_STATUS,
    crashAfterPublishBeforeFeedback: e.RECEIPT_TEST_STATUS,
    stableIdBrokerDeduplication: e.RECEIPT_TEST_STATUS,
    poisonTransactionBlocksCheckpoint: e.RECEIPT_TEST_STATUS,
    schemaDriftBlocksCheckpoint: e.RECEIPT_TEST_STATUS,
  },
  cleanup: {status: e.RECEIPT_CLEANUP, composeProject: e.RECEIPT_PROJECT},
  sha256: {
    sourceBinding: e.RECEIPT_BINDING,
    testLog: e.RECEIPT_TEST_LOG_HASH,
    serverLog: e.RECEIPT_SERVER_LOG_HASH,
  },
  credentials: 'disposable fixture credentials removed with the owned Compose project',
};
fs.writeFileSync(e.RECEIPT_PATH, `${JSON.stringify(receipt, null, 2)}\n`, {mode: 0o600});
NODE
  echo "CDC integration receipt: $RECEIPT"
}

cleanup() {
  local exit_status=$?
  trap - EXIT
  set +e
  if [[ "$COMPOSE_STARTED" == "1" ]]; then
    compose logs --no-color >"$OUTPUT_DIR/server.log" 2>&1
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
  export FRF_IGGY_IMAGE="${FRF_IGGY_IMAGE:-$PINNED_IGGY_IMAGE}"
  export FRF_POSTGRES_IMAGE="${FRF_POSTGRES_IMAGE:-$PINNED_POSTGRES_IMAGE}"
  [[ "$FRF_IGGY_IMAGE" == "$PINNED_IGGY_IMAGE" ]] || fail "Iggy image differs from the qualified digest"
  [[ "$FRF_POSTGRES_IMAGE" == "$PINNED_POSTGRES_IMAGE" ]] || fail "PostgreSQL image differs from the qualified digest"
  local credentials
  credentials="$(node -e "const {randomBytes}=require('node:crypto'); console.log('c008'+randomBytes(6).toString('hex')+':'+randomBytes(24).toString('hex'))")"
  export FRF_IGGY_USERNAME="${credentials%%:*}"
  export FRF_IGGY_PASSWORD="${credentials#*:}"
  export FRF_CDC_POSTGRES_USER="c008"
  export FRF_CDC_POSTGRES_PASSWORD="$(node -e "console.log(require('node:crypto').randomBytes(24).toString('hex'))")"
  export FRF_CDC_POSTGRES_DB="c008"
  compose config -q
  if [[ "${1:-}" == "--preflight-only" ]]; then
    echo "CDC integration preflight passed"
    return
  fi
  [[ $# == 0 ]] || fail "unknown argument: $1"

  mkdir -p "$OUTPUT_DIR"
  trap cleanup EXIT
  COMPOSE_STARTED=1
  compose up -d >"$OUTPUT_DIR/compose.log" 2>&1
  local iggy_id postgres_id mapping
  iggy_id="$(compose ps -q iggy)"
  postgres_id="$(compose ps -q postgres)"
  wait_for_services "$iggy_id" "$postgres_id"
  mapping="$(compose port iggy 8090)"
  IGGY_PORT="${mapping##*:}"
  mapping="$(compose port postgres 5432)"
  POSTGRES_PORT="${mapping##*:}"
  [[ "$IGGY_PORT" =~ ^[1-9][0-9]*$ && "$POSTGRES_PORT" =~ ^[1-9][0-9]*$ ]] ||
    fail "invalid dynamic fixture ports"
  IGGY_IMAGE_ID="$(docker inspect "$iggy_id" --format '{{.Image}}')"
  POSTGRES_IMAGE_ID="$(docker inspect "$postgres_id" --format '{{.Image}}')"
  export FRF_CDC_TEST_IGGY_URL="iggy://$FRF_IGGY_USERNAME:$FRF_IGGY_PASSWORD@127.0.0.1:$IGGY_PORT"
  export FRF_CDC_TEST_POSTGRES_URL="postgresql://$FRF_CDC_POSTGRES_USER:$FRF_CDC_POSTGRES_PASSWORD@127.0.0.1:$POSTGRES_PORT/$FRF_CDC_POSTGRES_DB"
  if CARGO_TARGET_DIR='/tmp/frf-c008-target' \
    CARGO_BUILD_BUILD_DIR='/tmp/frf-c008-build/{workspace-path-hash}' \
    RUSTC_WRAPPER='' \
    cargo test --offline -p frf-gateway --test cdc_integration \
      cdc_commit_mapping_contract --locked -- --ignored --exact --nocapture \
      >"$OUTPUT_DIR/test.log" 2>&1; then
    TEST_STATUS="pass"
  else
    tail -120 "$OUTPUT_DIR/test.log" >&2
    fail "CDC contract test failed"
  fi
  grep -q "CDC_COMMIT_MAPPING_PASS" "$OUTPUT_DIR/test.log" || fail "acceptance marker absent"
  grep -q "test result: ok. 1 passed; 0 failed; 0 ignored" "$OUTPUT_DIR/test.log" ||
    fail "CDC contract test was skipped or did not run exactly once"
}

main "$@"
