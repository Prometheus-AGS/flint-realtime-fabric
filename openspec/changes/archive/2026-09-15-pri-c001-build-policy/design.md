# Design — pri-c001-build-policy

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Fix the demonstrated missing SignalEnvelope subject and reconcile MSRV. Remove CI-reachable test execution while preserving local entry points; correct stale Dart-generator guidance to ADR-003.

Owner: Fabric. Legacy package: C01.
Dependencies: none.
Recommended agent: Codex.
Est. complexity: M; Complexity score: Medium.
Model class: frontier (project model policy is absent).

## Affected paths

- `Cargo.toml`
- `.github/workflows/ci.yml`
- `dagger/`
- `crates/frf-gateway/src/routes/dev.rs`
- `openspec/config.yaml`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Default, dev-endpoints and shape-facade checks plus applicable Clippy, format and SDK typecheck succeed locally against recorded source hashes.

B. Inventory every workflow/Dagger call path: CI performs build/lint/typecheck/format/package only; each removed runtime gate has a documented local invocation and is exercised by its owning later change.

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
