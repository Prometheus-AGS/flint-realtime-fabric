# Design — pri-c011-forge-watch

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Use the tonic Rust client generated/exported and compile-checked in c010 and canonical payload mapping. Deliver the authorized RLS re-query projection; implement the accepted safe deletion/invalidation contract and truncation reconstruction. Retain LISTEN default until parity passes.

Owner: Forge realtime/app/gateway. Legacy package: C06.
Dependencies: pri-c005-authority-lifetime, pri-c010-type-watch.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `../flint-forge/crates/fdb-realtime/src/`
- `../flint-forge/crates/fdb-app/src/lib.rs`
- `../flint-forge/crates/fdb-gateway/src/realtime_source.rs`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. A real SQL write reaches an authenticated GraphQL subscription with only permitted fields; denied rows, composite/key changes, deletes and truncated events behave as specified.

B. Disconnect/reconnect, revocation, backend failure and switch/rollback are explicit and do not silently lose updates or double-apply them. OQ-FRF-1 closes only after this receipt passes.

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
