# Design — pri-c023-release-signoff

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Re-audit effective source/configuration, verify all selected profile receipts and publish a dated local GO/NO-GO matrix. Reconcile capability, rollout/rollback and support documentation; retain separate phase, certification and publication statuses.

Owner: Fabric release + all consumer owners. Legacy package: C15.
Dependencies: pri-c001-build-policy, pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c006-watch-contract, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection, pri-c010-type-watch, pri-c011-forge-watch, pri-c012-sdk-pem-network, pri-c013-sync-persistence, pri-c014-aso-memory, pri-c015-aso-protected-proof, pri-c016-operations-recovery, pri-c017-hosted-media, pri-c018-matrix-recovery, pri-c019-atproto-recovery, pri-c020-sovereign-decode, pri-c021-admin-auth, pri-c022-platform-parity.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `docs/`
- `SECURITY.md`
- `CHANGELOG.md`
- `.kbd-orchestrator/phases/production-readiness-integration/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Every selected profile has exact repository/diff/lockfile/package/image/config/topology fingerprints, commands, timestamps, exits, scenario counts and artifact hashes; critical/high findings are resolved by an independent reviewer.

B. A narrower profile may receive a scoped interim verdict while other capabilities remain blocked; c023 and the whole phase cannot close until all agreed phase goals are satisfied or the operator explicitly accepts scope reduction. No registry publication/deployment is inferred.

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
