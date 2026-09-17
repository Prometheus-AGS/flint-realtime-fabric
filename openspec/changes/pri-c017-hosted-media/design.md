# Design — pri-c017-hosted-media

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q1, Q2 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Reuse researched LiveKit integration; prove real two-participant decoded audio/video and grant lifetime on the selected server version. Qualify relay only if advertised.

Owner: Fabric LiveKit + deployment. Legacy package: C11.
Dependencies: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-media-livekit/src/`
- `crates/frf-gateway/src/`
- `scripts/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Two clients show progressing decoded audio/video across late join and reconnect; wrong room/tenant/subject is denied and revoked participants stop under the accepted lifetime contract.

B. Any advertised cross-node relay passes a two-gateway campaign; absent configurations fail visibly. Local protocol fixtures alone cannot certify an untested production service version.

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
