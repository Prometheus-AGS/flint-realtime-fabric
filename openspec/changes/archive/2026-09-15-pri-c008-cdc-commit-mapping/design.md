# Design — pri-c008-cdc-commit-mapping

## Status and boundaries

Implemented design record. Read the phase plan, assessment, analysis and the
owning repository's AGENTS.md/CLAUDE.md with its source-bound evidence.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Reuse the pinned WAL parser; implement validated catalog/replica-identity mapping, schema-qualified routing and supported typed values. Couple acknowledged WAL progress to durable committed publication, including decode/transaction failures.

Owner: Fabric CDC. Legacy package: C05.
Dependencies: pri-c002-local-fixtures, pri-c006-watch-contract, pri-c007-broker-replay.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-postgres-cdc/src/consumer.rs`
- `crates/frf-postgres-cdc/src/decode.rs`
- `crates/frf-gateway/tests/cdc_integration.rs`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Real INSERT/UPDATE/DELETE and rollback/multi-row transactions exercise enrolled non-first/composite/non-UUID keys and accepted tenant modes; unsupported mappings fail enrollment, not silent delivery.

B. Crash between publication and LSN advancement permits defined deduplication without loss; poison rows, schema change, unchanged TOAST and missing old-key data cannot be skipped beneath an acknowledged checkpoint.

## Compatibility and rollout

Keep proto/flint/v1 unchanged and use the c006 accepted versioned contract where
applicable. Codegen/ADR versions do not change implicitly. Retain LISTEN and ASO
experimental gating until their owning functional proofs pass. Record forward and
rollback behavior for touched config/data/package boundaries before rollout.
If research demonstrates infeasibility, split/revise the proposal and preserve
its open acceptance obligation; diagnosis alone cannot complete implementation.

## Evidence and local verification

Use isolated synthetic fixtures and real dependencies. Runtime tests run locally;
CI only builds/lints/typechecks/formats/packages. Missing prerequisites, skipped
required cases, empty streams and loopback substitution are failures. Capture
source/dirty-diff, locks, generated/package/image/config digests, topology,
commands, times, exits, expected scenarios and output hashes in a source-bound
receipt at the phase evidence path for this change. Documentation-only decisions
use recorded source evidence and operator provenance rather than invented tests.
The 500-line maintainability limit applies to hand-authored files; generated
dependency lockfiles remain in their package manager's required native format.

## Library reuse

Reuse these Analyze candidates, subject to their recorded risks. Evidence is copied verbatim:

```json
[
  {
    "id": "cand-001",
    "name": "Existing Apache Iggy fork behind LogBroker",
    "kind": "library",
    "repo_url": "https://github.com/GQAdonis/iggy",
    "registry": "none",
    "fit_for_gap": "F01/F03: durable event history, replay and projection population",
    "verdict": "adapt",
    "license": "Apache-2.0",
    "evidence": [
      {
        "tier": 1,
        "source_url": "https://github.com/apache/iggy",
        "claim": "Active upstream found; this does not certify the pinned fork."
      },
      {
        "tier": 2,
        "source_url": "https://iggy.apache.org/docs/server/schema",
        "claim": "Polling supports explicit offsets and separately stored consumer offsets."
      },
      {
        "tier": 2,
        "source_url": "https://iggy.apache.org/docs/introduction/getting-started",
        "claim": "Polling auto-commit is not an end-to-end durable-processing acknowledgement."
      }
    ],
    "risks": [
      "Pinned fork d34b9c96 must be qualified; upstream documentation is not a fork compatibility receipt.",
      "FRF returns producer-supplied offsets, decodes payload without broker offset metadata, and leaves polling auto-commit enabled by default.",
      "Retention, partition scope, producer deduplication, fsync/ack behavior and replay authorization remain design/proof obligations."
    ],
    "decision_rationale": "Keep the decided broker. Adapt its existing port implementation and compose a separate entity projection adapter; do not add a parallel broker."
  },
  {
    "id": "cand-002",
    "name": "pg_walstream 0.6.3",
    "kind": "library",
    "repo_url": "https://github.com/isdaniel/pg-walstream",
    "registry": "crates",
    "fit_for_gap": "F03: committed WAL ingestion, typed/schema/key mapping and checkpoints",
    "verdict": "adapt",
    "license": "BSD-3-Clause",
    "evidence": [
      {
        "tier": 1,
        "source_url": "https://github.com/isdaniel/pg-walstream",
        "claim": "Exact Rust WAL library found; local Cargo.lock resolves 0.6.3."
      },
      {
        "tier": 2,
        "source_url": "https://github.com/isdaniel/pg-walstream",
        "claim": "Locally installed 0.6.3 protocol source exposes namespace, column metadata, replica-identity key flags and applied-LSN state; precise excerpts and hashes are in analyze-source-excerpts.json. Context7 returned no matching library; remote docs and registry lookup failed."
      }
    ],
    "risks": [
      "Current registry maintenance and latest release unverified.",
      "Replica-identity columns are not always the application primary key; initial metadata availability on high-level events must be proven or use a catalog lookup.",
      "FRF ignores transaction/control events and skips decode failures; define committed transaction checkpointing, poison-event handling, unchanged TOAST and schema evolution before any no-loss claim."
    ],
    "decision_rationale": "Conditional adapt recommendation only: retain the existing parser for design, but do not qualify it for production until current maintenance/security status and exact-version API fit are checked. Build FRF mapping and transaction-aware durability around it if those checks pass."
  }
]
```
