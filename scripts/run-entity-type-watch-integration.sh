#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_RUNNER="$ROOT_DIR/scripts/run-entity-projection-integration.sh"
GENERATED_RUNNER="$(mktemp "$ROOT_DIR/scripts/.entity-type-watch-runner.XXXXXX")"

cleanup() {
  rm -f "$GENERATED_RUNNER"
}
trap cleanup EXIT

BASE_RUNNER="$BASE_RUNNER" GENERATED_RUNNER="$GENERATED_RUNNER" node <<'NODE'
const fs = require('node:fs');
let source = fs.readFileSync(process.env.BASE_RUNNER, 'utf8');
const replacements = [
  [
    'OUTPUT_DIR="$ROOT_DIR/target/entity-projection-receipts/$RUN_ID"',
    'OUTPUT_DIR="$ROOT_DIR/target/entity-type-watch-receipts/$RUN_ID"',
  ],
  [
    'SOURCE_MANIFEST="$ROOT_DIR/openspec/changes/pri-c009-entity-projection/files.txt"',
    'SOURCE_MANIFEST="$ROOT_DIR/openspec/changes/pri-c010-type-watch/files.txt"',
  ],
  [
    "-path '*-pri-c009-entity-projection/files.txt'",
    "-path '*-pri-c010-type-watch/files.txt'",
  ],
  [
    'scenarios: {databaseToGetEntity: status, databaseToWatchEntity: status, backlogReadiness: status, snapshotWalOverlap: status, restartCursorCoherence: status, deleteClearsState: status, perEventAuthorization: status, transactionCancellation: status},',
    'scenarios: {postgresCommitToTonicWatch: status, twoAuthorizedSubscribers: status, durableTypedSnapshot: status, resumeAfterCheckpoint: status, keyOnlyDelete: status, sameTenantUnauthorizedDenied: status},',
  ],
  [
    "CARGO_TARGET_DIR=/tmp/frf-c009-target CARGO_BUILD_BUILD_DIR='/tmp/frf-c009-build/{workspace-path-hash}'",
    "CARGO_TARGET_DIR=/tmp/frf-c010-target CARGO_BUILD_BUILD_DIR='/tmp/frf-c010-build/{workspace-path-hash}'",
  ],
  ['--test entity_projection_integration', '--test entity_type_watch_integration'],
  [
    'committed_database_changes_reach_durable_v1_entity_reads_and_watches',
    'committed_database_changes_cross_the_v2_tonic_watch_boundary',
  ],
  ['grep -q "ENTITY_PROJECTION_PASS"', 'grep -q "ENTITY_TYPE_WATCH_PASS"'],
  ['Entity projection receipt:', 'Entity type watch receipt:'],
  ['Entity projection preflight passed', 'Entity type watch preflight passed'],
];
for (const [before, after] of replacements) {
  const occurrences = source.split(before).length - 1;
  if (occurrences !== 1) {
    throw new Error(`base runner contract changed for replacement: ${before}`);
  }
  source = source.replace(before, after);
}
fs.writeFileSync(process.env.GENERATED_RUNNER, source, {mode: 0o700});
NODE

bash "$GENERATED_RUNNER" "$@"
