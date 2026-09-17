# Design — pri-c015-aso-protected-proof

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q2, Q7 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Prove current-source protected replication using the existing shape feed and materializer, independently of generic entity watching. Cover every selected browser/native topology and approved schema/egress boundary.

Owner: ASO + Gate/Fabric + PEM. Legacy package: C08.
Dependencies: pri-c005-authority-lifetime, pri-c014-aso-memory.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/`
- `crates/frf-app/src/shape/`
- `crates/frf-gateway/src/routes/shape.rs`
- `../flint-gate/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Crash before SQL checkpoint rolls back rows/checkpoints; crash after SQL commit before PEM publication recovers coherently only after current authorization. Logout/practice switch fences stale owners and clears the old graph.

B. Interrupted shape bodies, expiry/refetch, revocation timing, direct-backend isolation and forbidden/local-only field egress pass for each selected topology; no competing generic-watch writer handles the same clinical rows. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.

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
    "id": "cand-003",
    "name": "PGlite and existing ASO materializer",
    "kind": "library",
    "repo_url": "https://github.com/electric-sql/pglite",
    "registry": "npm",
    "fit_for_gap": "F07/F11: protected replica lifecycle and browser memory qualification",
    "verdict": "adapt",
    "license": "Apache-2.0",
    "maintenance": {
      "last_release": "0.5.8",
      "registry_modified": "2026-08-26T18:40:09.299Z"
    },
    "evidence": [
      {
        "tier": 1,
        "source_url": "https://github.com/electric-sql/pglite",
        "claim": "Existing embedded Postgres implementation found."
      },
      {
        "tier": 2,
        "source_url": "https://github.com/electric-sql/pglite/blob/main/docs/docs/api.md",
        "claim": "close() provides database shutdown; it is not a measured browser RSS guarantee."
      },
      {
        "tier": 2,
        "source_url": "https://github.com/electric-sql/pglite/blob/main/docs/docs/multi-tab-worker.md",
        "claim": "Worker integration exists; ownership and disposal must fit the existing ASO fencing model."
      },
      {
        "tier": 3,
        "source_url": "https://registry.npmjs.org/@electric-sql%2Fpglite",
        "claim": "Registry latest is 0.5.8; ASO declares ^0.5.8. A range is not proof of the installed release identity."
      }
    ],
    "risks": [
      "Existing measured incremental RSS exceeds 512 MiB; lifecycle API availability does not demonstrate a fix.",
      "Profile wasm/database allocation, graph copies, batch sizes, worker count and measurement scope before selecting an optimization.",
      "Browser behavior does not qualify native storage or credential boundaries."
    ],
    "decision_rationale": "Reuse implemented materializer, atomic checkpointing and owner fences. Diagnose and qualify memory before considering a storage replacement or changing budgets."
  }
]
```
