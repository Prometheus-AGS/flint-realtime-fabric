# Design — pri-c020-sovereign-decode

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q1, Q2 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Reproduce the zero-decoded-frame failure in a locally controlled Linux ICE/TURN topology. Trace room/track/keyframe routing, apply the smallest demonstrated correction and prove room-scoped protected egress.

Owner: Fabric str0m media. Legacy package: C13.
Dependencies: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-media-str0m/src/`
- `crates/frf-gateway/src/`
- `scripts/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Repeated two-peer campaigns show increasing receiver framesDecoded plus audio progress, including late join/reconnect and multiple rooms; ICE/RTP alone is insufficient.

B. Revocation/expiry removes protected fan-out within c003's accepted bound; wrong-room/tenant traffic is denied and task/socket counts return to baseline. Keep production mode gated until proof.

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
