#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixturePath = resolve(root, "proto/flint/v2/watch-contract-fixtures.json");
const protoPath = resolve(root, "proto/flint/v2/entity.proto");

const expectedV1 = new Map([
  ["agent.proto", "d5908e755d653c05665c632a097a293ead828429362adaf60af78200cf8e4220"],
  ["authz.proto", "258f2b3676ce39b435492b63f88813dba4090a34e660ebaf116f9f288d4efdce"],
  ["entity.proto", "f90e93b6a806039603a76bff9e47d359838cda97f5fb0b418009c16a22fbbcda"],
  ["envelope.proto", "dac772037b2895fef78fa980971d256570e6d9e21502bde2ede926b99c73384c"],
  ["signal.proto", "39ffc96405e9c972fb49d90d01de47508fa72ed24ab59c08556c74bd30daaeba"],
  ["sync.proto", "47925c6e1a3466abd8aed2eed32af82f67eb85daf5eea54ba6e0e49e1bbf8662"],
]);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function fail(message) {
  throw new Error(`watch contract: ${message}`);
}

const fixture = JSON.parse(await readFile(fixturePath, "utf8"));
const proto = await readFile(protoPath, "utf8");

for (const [name, expected] of expectedV1) {
  const bytes = await readFile(resolve(root, "proto/flint/v1", name));
  if (sha256(bytes) !== expected) fail(`frozen v1 hash changed: ${name}`);
}

const keyByName = new Map();
for (const keyCase of fixture.canonicalKeys) {
  const canonical = canonicalJson(keyCase.parts);
  const encoded = Buffer.from(canonical).toString("base64url");
  const expected = `frfkey:v1:${encoded}`;
  if (keyCase.canonicalId !== expected) fail(`canonical key mismatch: ${keyCase.name}`);
  if (keyCase.primaryKeyOrder.join("\0") !== keyCase.parts.map((part) => part.column).join("\0")) {
    fail(`primary-key ordinal mismatch: ${keyCase.name}`);
  }
  keyByName.set(keyCase.name, keyCase);
}

let priorIndex = -1;
for (const event of fixture.committedTransaction.events) {
  if (event.transactionIndex !== priorIndex + 1) fail("transaction indices are not contiguous");
  priorIndex = event.transactionIndex;
  const key = keyByName.get(event.keyCase);
  if (!key) fail(`unknown key fixture: ${event.keyCase}`);
  const identity = {
    commit_lsn: fixture.committedTransaction.commitLsn,
    entity_type: event.entityType,
    epoch: fixture.committedTransaction.epoch,
    key: key.canonicalId,
    transaction_index: event.transactionIndex,
  };
  const expected = `frfevent:v1:${sha256(canonicalJson(identity))}`;
  if (event.eventId !== expected) fail(`stable event identity mismatch at index ${event.transactionIndex}`);
  if (event.op === "delete" && event.recordRequired) fail("delete fixture requires historical row data");
}

if (fixture.committedTransaction.publishedBeforeCommit !== false) {
  fail("transaction fixture permits pre-commit publication");
}
if (fixture.committedTransaction.poisonEventBlocksCheckpoint !== true) {
  fail("transaction fixture permits checkpointing past a poison event");
}

const requiredSnapshotOrder = [
  "accepted-with-barrier",
  "snapshot-row-zero-or-more",
  "snapshot-complete-same-barrier",
  "mutation-strictly-after-barrier",
];
if (fixture.snapshotToLive.orderedFrames.join("\0") !== requiredSnapshotOrder.join("\0")) {
  fail("snapshot-to-live barrier sequence changed");
}
if (!fixture.snapshotToLive.duplicatesAllowed || fixture.snapshotToLive.deduplicateBy !== "event_id") {
  fail("at-least-once snapshot/live deduplication contract changed");
}

const expectedResumeResults = new Set([
  "accepted-after-checkpoint",
  "resnapshot_required:history_expired",
  "resnapshot_required:source_epoch_changed",
  "resnapshot_required:checkpoint_scope_mismatch",
]);
for (const resumeCase of fixture.resumeCases) expectedResumeResults.delete(resumeCase.result);
if (expectedResumeResults.size !== 0 || fixture.resumeCases.length !== 4) {
  fail("resume/replay matrix is incomplete");
}

if (fixture.projection.clientCanSelectFields !== false) fail("client-selected projection is enabled");
const projectedOrder = fixture.projection.allowedFields.map((field) => field.column);
const sourceOrder = fixture.projection.sourceColumnOrder.filter((column) => projectedOrder.includes(column));
if (projectedOrder.join("\0") !== sourceOrder.join("\0")) fail("projection fields lost source ordinal order");
for (const excluded of fixture.projection.excludedFields) {
  if (projectedOrder.includes(excluded)) fail(`excluded projection field is present: ${excluded}`);
}
if (
  fixture.authorizedDelete.authorizationResource !== "canonical-key" ||
  fixture.authorizedDelete.recordPresent ||
  fixture.authorizedDelete.previousRecordPresent ||
  !fixture.authorizedDelete.entityTypePresent ||
  !fixture.authorizedDelete.tenantIdPresent ||
  !fixture.authorizedDelete.checkpointAfterAuthorization
) {
  fail("authorized key-only delete contract changed");
}

const mutationBlock = proto.match(/message EntityMutation \{([\s\S]*?)\n\}/)?.[1] ?? "";
if (!mutationBlock.includes("EntityType entity_type") || !mutationBlock.includes("string tenant_id")) {
  fail("mutation frames are not self-describing by entity type and tenant");
}

for (const field of fixture.unauthorizedHistory.forbiddenProgressFields) {
  const block = proto.match(/message CheckpointAdvanced \{([\s\S]*?)\n\}/)?.[1] ?? "";
  if (block.includes(field)) fail(`checkpoint progress leaks ${field}`);
}

if (!proto.includes("package flint.v2;")) fail("v2 package is missing");
if (!proto.includes("rpc WatchEntityType(WatchEntityTypeRequest) returns (stream WatchEntityTypeResponse);")) {
  fail("WatchEntityType streaming RPC is missing");
}
if (fixture.lag.silentDrop !== false) fail("lag fixture permits silent drop");
if (fixture.cancellation.framesProducedAfterObservation !== 0) {
  fail("cancellation fixture permits post-cancel production");
}

console.log(
  `watch contract passed: ${expectedV1.size} frozen v1 files, ${fixture.canonicalKeys.length} key shapes, ${fixture.resumeCases.length} resume cases`,
);
