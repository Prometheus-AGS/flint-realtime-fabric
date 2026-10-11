# Design — pri-c005-authority-lifetime

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q2, Q5, Q7 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Verify real Gate credentials and current object authorization on generic streams, and preserve Gate/ASO fresh authority on restricted shape streams. Check expiry, rotation, revoked grants, cancellation and bypass paths.

Owner: Gate + Fabric; Forge/ASO boundary owners. Legacy package: C03.
Dependencies: pri-c002-local-fixtures, pri-c003-scope-research-contract, pri-c004-deployment-profiles.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-gateway/src/`
- `crates/frf-app/src/`
- `crates/frf-authz-keto/src/`
- `../flint-gate/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Real Gate-issued tokens allow only intended issuer/audience/tenant/subject and object; same-tenant unauthorized subjects, cross-tenant access, authority loss, key rotation and stale refill races are exercised locally.

B. ASO authoritative commit/expiry to last server-produced protected frame or cancellation preventing the next frame is at most 5000 ms, followed by new-request denial; direct Kratos revocation uses the documented observation/ASO-denial start. All other lanes meet the lifetime contract accepted in c003.

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
