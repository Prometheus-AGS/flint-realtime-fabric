#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

RUN_ID="$(date -u +%Y%m%d%H%M%S)-$$"
PROJECT="frf-projection-${RUN_ID}"
OUTPUT_DIR="$ROOT_DIR/target/entity-projection-receipts/$RUN_ID"
RECEIPT="$OUTPUT_DIR/receipt.json"
STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
PINNED_IGGY_IMAGE="iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5"
PINNED_POSTGRES_IMAGE="postgres@sha256:18cfe3ef5e6815560c98237d6216d1e5119702fb0f3894c8785dd58b8bbe5d73"
PINNED_SURREAL_IMAGE="surrealdb/surrealdb@sha256:408c6930d730adcf65d4bf91f385381e75d9db32663a33ee1b026c4d43914732"
COMPOSE_STARTED=0
TEST_STATUS="fail"
CLEANUP_STATUS="not-started"
IGGY_PORT=""
POSTGRES_PORT=""
SURREAL_PORT=""
IGGY_IMAGE_ID=""
POSTGRES_IMAGE_ID=""
SURREAL_IMAGE_ID=""
SOURCE_MANIFEST="$ROOT_DIR/openspec/changes/pri-c009-entity-projection/files.txt"
if [[ ! -f "$SOURCE_MANIFEST" ]]; then
  SOURCE_MANIFEST="$(find "$ROOT_DIR/openspec/changes/archive" -maxdepth 2 \
    -path '*-pri-c009-entity-projection/files.txt' -print | sort | tail -1)"
fi

compose() {
  docker compose --project-name "$PROJECT" \
    -f compose.integration.yml -f compose.entity-projection-integration.yml "$@"
}

fail() {
  echo "entity-projection integration: $*" >&2
  exit 1
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

require_prerequisites() {
  command -v docker >/dev/null || fail "docker is required"
  command -v cargo >/dev/null || fail "cargo is required"
  command -v node >/dev/null || fail "node is required"
  docker info >/dev/null 2>&1 || fail "docker daemon is unavailable"
  [[ -f "$SOURCE_MANIFEST" ]] || fail "c009 source manifest is unavailable"
}

assert_owned_project() {
  [[ "$PROJECT" =~ ^frf-projection-[0-9]{14}-[0-9]+$ ]] || fail "unsafe compose project"
}

wait_for_services() {
  PROJECTION_POSTGRES_PORT="$1" PROJECTION_IGGY_PORT="$2" PROJECTION_SURREAL_PORT="$3" node <<'NODE'
const net = require('node:net');

const endpoints = [
  ['PostgreSQL', Number(process.env.PROJECTION_POSTGRES_PORT)],
  ['Iggy', Number(process.env.PROJECTION_IGGY_PORT)],
  ['SurrealDB', Number(process.env.PROJECTION_SURREAL_PORT)],
];
const deadline = Date.now() + 60_000;

function connect(port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({host: '127.0.0.1', port});
    const finish = (ready) => {
      socket.destroy();
      resolve(ready);
    };
    socket.setTimeout(1_000, () => finish(false));
    socket.once('connect', () => finish(true));
    socket.once('error', () => finish(false));
  });
}

(async () => {
  while (Date.now() < deadline) {
    const states = await Promise.all(endpoints.map(([, port]) => connect(port)));
    if (states.every(Boolean)) process.exit(0);
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  const states = await Promise.all(endpoints.map(([, port]) => connect(port)));
  const unavailable = endpoints.filter((_, index) => !states[index]).map(([name]) => name);
  console.error(`fixture readiness timed out: ${unavailable.join(', ')}`);
  process.exit(1);
})();
NODE
}

write_receipt() {
  local exit_status="$1"
  local binding_file="$OUTPUT_DIR/source-binding.sha256"
  : >"$binding_file"
  while IFS= read -r path; do
    case "$path" in
      .kbd-orchestrator/*|.refiner/*|openspec/*) continue ;;
    esac
    [[ -f "$ROOT_DIR/$path" ]] || continue
    printf '%s  %s\n' "$(sha256_file "$ROOT_DIR/$path")" "$path" >>"$binding_file"
  done <"$SOURCE_MANIFEST"
  local binding_hash
  binding_hash="$(sha256_file "$binding_file")"
  RECEIPT_PATH="$RECEIPT" RECEIPT_RUN_ID="$RUN_ID" RECEIPT_STARTED="$STARTED_AT" \
  RECEIPT_FINISHED="$(date -u +%Y-%m-%dT%H:%M:%SZ)" RECEIPT_EXIT="$exit_status" \
  RECEIPT_PROJECT="$PROJECT" RECEIPT_COMMIT="$(git rev-parse HEAD)" \
  RECEIPT_BINDING="$binding_hash" RECEIPT_TEST_STATUS="$TEST_STATUS" \
  RECEIPT_CLEANUP="$CLEANUP_STATUS" RECEIPT_IGGY_PORT="$IGGY_PORT" \
  RECEIPT_POSTGRES_PORT="$POSTGRES_PORT" RECEIPT_SURREAL_PORT="$SURREAL_PORT" \
  RECEIPT_IGGY_ID="$IGGY_IMAGE_ID" RECEIPT_POSTGRES_ID="$POSTGRES_IMAGE_ID" \
  RECEIPT_SURREAL_ID="$SURREAL_IMAGE_ID" \
  RECEIPT_TEST_HASH="$(sha256_file "$OUTPUT_DIR/test.log")" \
  RECEIPT_SERVER_HASH="$(sha256_file "$OUTPUT_DIR/server.log")" node <<'NODE'
const fs = require('node:fs');
const e = process.env;
const status = e.RECEIPT_TEST_STATUS;
const receipt = {
  schemaVersion: 1,
  runId: e.RECEIPT_RUN_ID,
  source: {commit: e.RECEIPT_COMMIT, candidateArtifactSetSha256: e.RECEIPT_BINDING},
  startedAt: e.RECEIPT_STARTED,
  finishedAt: e.RECEIPT_FINISHED,
  exitStatus: Number(e.RECEIPT_EXIT),
  topology: {
    composeProject: e.RECEIPT_PROJECT,
    endpoints: {iggy: `127.0.0.1:${e.RECEIPT_IGGY_PORT}`, postgres: `127.0.0.1:${e.RECEIPT_POSTGRES_PORT}`, surreal: `127.0.0.1:${e.RECEIPT_SURREAL_PORT}`},
    imageIds: {iggy: e.RECEIPT_IGGY_ID, postgres: e.RECEIPT_POSTGRES_ID, surreal: e.RECEIPT_SURREAL_ID},
  },
  scenarios: {databaseToGetEntity: status, databaseToWatchEntity: status, backlogReadiness: status, snapshotWalOverlap: status, restartCursorCoherence: status, deleteClearsState: status, perEventAuthorization: status, transactionCancellation: status},
  cleanup: {status: e.RECEIPT_CLEANUP, composeProject: e.RECEIPT_PROJECT},
  sha256: {sourceBinding: e.RECEIPT_BINDING, testLog: e.RECEIPT_TEST_HASH, serverLog: e.RECEIPT_SERVER_HASH},
  credentials: 'per-run fixture credentials removed with the owned Compose project',
};
fs.writeFileSync(e.RECEIPT_PATH, `${JSON.stringify(receipt, null, 2)}\n`, {mode: 0o600});
NODE
}

cleanup() {
  local exit_status=$?
  trap - EXIT
  set +e
  if [[ "$COMPOSE_STARTED" == "1" ]]; then
    compose logs --no-color >"$OUTPUT_DIR/server.log" 2>&1
    if assert_owned_project && compose down -v --remove-orphans >/dev/null 2>&1; then
      CLEANUP_STATUS="owned-resources-removed"
    else
      CLEANUP_STATUS="compose-down-failed"
      exit_status=1
    fi
  fi
  write_receipt "$exit_status"
  echo "Entity projection receipt: $RECEIPT"
  exit "$exit_status"
}

main() {
  require_prerequisites
  assert_owned_project
  export FRF_IGGY_IMAGE="${FRF_IGGY_IMAGE:-$PINNED_IGGY_IMAGE}"
  export FRF_POSTGRES_IMAGE="${FRF_POSTGRES_IMAGE:-$PINNED_POSTGRES_IMAGE}"
  export FRF_SURREAL_IMAGE="${FRF_SURREAL_IMAGE:-$PINNED_SURREAL_IMAGE}"
  [[ "$FRF_IGGY_IMAGE" == "$PINNED_IGGY_IMAGE" ]] || fail "Iggy image differs from qualified digest"
  [[ "$FRF_POSTGRES_IMAGE" == "$PINNED_POSTGRES_IMAGE" ]] || fail "PostgreSQL image differs from qualified digest"
  [[ "$FRF_SURREAL_IMAGE" == "$PINNED_SURREAL_IMAGE" ]] || fail "SurrealDB image differs from qualified digest"
  local random
  random="$(node -e "console.log(require('node:crypto').randomBytes(18).toString('hex'))")"
  export FRF_IGGY_USERNAME="c009${random:0:8}"
  export FRF_IGGY_PASSWORD="${random}iggy"
  export FRF_PROJECTION_POSTGRES_USER="c009"
  export FRF_PROJECTION_POSTGRES_PASSWORD="${random}postgres"
  export FRF_PROJECTION_POSTGRES_DB="c009"
  export FRF_PROJECTION_SURREAL_USER="root"
  export FRF_PROJECTION_SURREAL_PASSWORD="${random}surreal"
  compose config -q
  [[ "${1:-}" != "--preflight-only" ]] || { echo "Entity projection preflight passed"; return; }
  [[ $# == 0 ]] || fail "unknown argument: $1"

  mkdir -p "$OUTPUT_DIR"
  : >"$OUTPUT_DIR/test.log"
  : >"$OUTPUT_DIR/server.log"
  trap cleanup EXIT
  COMPOSE_STARTED=1
  compose up -d >"$OUTPUT_DIR/compose.log" 2>&1
  local pg_id iggy_id surreal_id mapping
  pg_id="$(compose ps -q postgres)"
  iggy_id="$(compose ps -q iggy)"
  surreal_id="$(compose ps -q surreal)"
  mapping="$(compose port postgres 5432)"; POSTGRES_PORT="${mapping##*:}"
  mapping="$(compose port iggy 8090)"; IGGY_PORT="${mapping##*:}"
  mapping="$(compose port surreal 8000)"; SURREAL_PORT="${mapping##*:}"
  wait_for_services "$POSTGRES_PORT" "$IGGY_PORT" "$SURREAL_PORT"
  POSTGRES_IMAGE_ID="$(docker inspect "$pg_id" --format '{{.Image}}')"
  IGGY_IMAGE_ID="$(docker inspect "$iggy_id" --format '{{.Image}}')"
  SURREAL_IMAGE_ID="$(docker inspect "$surreal_id" --format '{{.Image}}')"
  export FRF_PROJECTION_TEST_POSTGRES_URL="postgresql://$FRF_PROJECTION_POSTGRES_USER:$FRF_PROJECTION_POSTGRES_PASSWORD@127.0.0.1:$POSTGRES_PORT/$FRF_PROJECTION_POSTGRES_DB"
  export FRF_PROJECTION_TEST_IGGY_URL="iggy://$FRF_IGGY_USERNAME:$FRF_IGGY_PASSWORD@127.0.0.1:$IGGY_PORT"
  export FRF_PROJECTION_TEST_SURREAL_URL="ws://127.0.0.1:$SURREAL_PORT"
  if CARGO_TARGET_DIR=/tmp/frf-c009-target CARGO_BUILD_BUILD_DIR='/tmp/frf-c009-build/{workspace-path-hash}' RUSTC_WRAPPER='' \
    cargo test --offline -p frf-gateway --test entity_projection_integration \
      committed_database_changes_reach_durable_v1_entity_reads_and_watches --locked -- --ignored --exact --nocapture \
      >"$OUTPUT_DIR/test.log" 2>&1; then
    TEST_STATUS="pass"
  else
    tail -160 "$OUTPUT_DIR/test.log" >&2
    fail "entity projection contract test failed"
  fi
  grep -q "ENTITY_PROJECTION_PASS" "$OUTPUT_DIR/test.log" || fail "acceptance marker absent"
  grep -q "test result: ok. 1 passed; 0 failed; 0 ignored" "$OUTPUT_DIR/test.log" || fail "acceptance test did not run exactly once"
}

main "$@"
