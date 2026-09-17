# Design — pri-c022-platform-parity

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q3 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Generate using pinned or explicitly superseded ADR-003 tooling. Address Dart async transport only through a demonstrated compatible approach; qualify every required install/runtime/device surface.

Owner: Fabric SDK/FFI + consumer owners. Legacy package: C14.
Dependencies: pri-c003-scope-research-contract, pri-c006-watch-contract, pri-c012-sdk-pem-network, pri-c013-sync-persistence.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-ffi/`
- `sdks/go/`
- `sdks/csharp/`
- `sdks/swift/`
- `sdks/kotlin/`
- `sdks/dart/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Each required platform installs the packaged artifact and performs authenticated publish/watch/reconnect; supported offline CRDT behavior survives process replacement.

B. Dart async remains blocked until a real transport proof passes; unsupported platforms remain excluded with explicit reasons, without claiming full phase closure or parity.

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

No new library is selected here. Existing dependencies require c003 qualification where applicable; this proposal is not evidence that unresearched APIs fit.
