# Design — pri-c016-operations-recovery

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q2, Q6 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Execute load and failure campaigns at accepted profile targets; cover broker/WAL/checkpoint lag, backup/restore, rolling replacement, secrets/certificate rotation, drains and no-progress alerts. Final qualification cannot close before every selected profile implementation and client surface is complete; early diagnostic campaigns are non-final.

Owner: Deployment + Fabric operations. Legacy package: C10.
Dependencies: pri-c003-scope-research-contract, pri-c004-deployment-profiles, pri-c005-authority-lifetime, pri-c007-broker-replay, pri-c008-cdc-commit-mapping, pri-c009-entity-projection, pri-c010-type-watch, pri-c011-forge-watch, pri-c012-sdk-pem-network, pri-c013-sync-persistence, pri-c014-aso-memory, pri-c015-aso-protected-proof, pri-c017-hosted-media, pri-c018-matrix-recovery, pri-c019-atproto-recovery, pri-c020-sovereign-decode, pri-c021-admin-auth, pri-c022-platform-parity.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `compose.yml`
- `k8s/`
- `scripts/`
- `docs/`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Local campaigns meet accepted subscription/change-rate/latency/retention/RTO/RPO targets with explicit resource allocation, no required skips and recorded recovery positions.

B. Restoration and credential/certificate rotation preserve required authorization and durable state; bounded backpressure/cancellation and alerts prevent false-ready/no-progress operation. Repeat affected receipts after consumer or profile changes.

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
