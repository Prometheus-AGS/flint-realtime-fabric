# Design — pri-c007-broker-replay

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Replace producer-local replay counters with the accepted broker position mapping. Configure explicit commit policy instead of polling-time defaults and prove exact pinned-fork seek/replay/ack semantics, stable producer identity and cancellation.

Owner: Fabric broker adapter. Legacy package: C05.
Dependencies: pri-c002-local-fixtures, pri-c006-watch-contract.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-broker-iggy/src/`
- `crates/frf-broker-iggy/tests/`
- `crates/frf-ports/src/log_broker.rs`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Crash after poll but before durable consumer checkpoint replays the event; restart after acknowledgement respects defined cursor inclusivity and duplicate rules.

B. Producer restart does not reuse a replay position; independent consumers, partition boundaries, retention expiry and cancellation behave as the frozen contract requires. Record actual server durability/flush configuration.

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
  }
]
```
