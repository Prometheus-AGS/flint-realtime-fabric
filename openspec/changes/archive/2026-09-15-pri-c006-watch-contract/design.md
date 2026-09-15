# Design — pri-c006-watch-contract

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q4, Q5, Q6 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Specify a new versioned WatchEntityType contract, canonical typed payload/key mapping and compatibility with immutable v1. Separate source LSN/epoch, stable event identity, partition offset and client checkpoint; specify snapshot barrier, ordering, duplicates, retention, lag, cancellation and authorized deletes.

Owner: Fabric contract + Forge/PEM consumers. Legacy package: C04.
Dependencies: pri-c003-scope-research-contract.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `proto/flint/`
- `crates/frf-domain/src/`
- `crates/frf-ports/src/`
- `docs/decisions/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Contract scenarios cover accepted schema/key/tenant shapes, transaction commit, snapshot-to-live races, restart/replay, old cursors, unauthorized history and deletion/projection; consumer acceptance is recorded before codegen.

B. Frozen proto/flint/v1 files remain byte-identical; new-version generation and old/new compatibility validation succeed. Semver impact on frf-domain/frf-ports and one-port adapter boundaries are documented.

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
  },
  {
    "id": "cand-004",
    "name": "Pinned Connect-ES 1.x and existing generated SDK pipeline",
    "kind": "library",
    "repo_url": "https://github.com/connectrpc/connect-es",
    "registry": "npm",
    "fit_for_gap": "F08/F03: generated v2 watch transport and real packaged consumer proof",
    "verdict": "adapt",
    "license": "Apache-2.0",
    "maintenance": {
      "last_release": "2.2.0 (registry latest; not selected)",
      "registry_modified": "2026-09-07T06:45:44.581Z"
    },
    "evidence": [
      {
        "tier": 1,
        "source_url": "https://github.com/connectrpc/connect-es",
        "claim": "Existing TypeScript RPC client family found."
      },
      {
        "tier": 2,
        "source_url": "https://github.com/connectrpc/connect-es/blob/main/MIGRATING.md",
        "claim": "Current docs show v2 changes to client construction and Protobuf-ES generation; they cannot be copied into the pinned v1 SDK."
      },
      {
        "tier": 3,
        "source_url": "https://registry.npmjs.org/@connectrpc%2Fconnect/1.6.1",
        "claim": "Pinned-line 1.6.1 metadata is available; latest lookup separately returns 2.2.0."
      }
    ],
    "risks": [
      "Exact v1 browser streaming/cancellation behavior needs installed-source inspection and packed network tests; current Context7 examples are mostly v2.",
      "CJS export is advertised without an emitted CJS artifact; fix build/export contract before compatibility signoff.",
      "RPC proto v2 does not require Connect-ES package v2. ADR-003 upgrades require an explicit superseding decision."
    ],
    "decision_rationale": "Retain ADR-003 and generated clients. Repair distribution and adapters using the current toolchain; avoid an unrelated major-version migration."
  }
]
```
