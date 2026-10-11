# Design — pri-c003-scope-research-contract

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Resolve Q1–Q7 with provenance and research the named identity/policy, persistence/deployment, media/federation and platform backlog in bounded passes. Inventory source/package/image revisions and runtime profiles. Record Q8 separately; KBD migration is not part of this change. Research and decision facilitation may start immediately; unresolved Q1–Q7 block closure and dependent implementation, not the start of this change.

Owner: Fabric coordinator + Gate/Forge/PEM/ASO owners. Legacy package: C01 + Analyze backlog.
Dependencies: none.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `.kbd-orchestrator/phases/production-readiness-integration/analysis.md`
- `.kbd-orchestrator/phases/production-readiness-integration/decision-log.md`
- `.kbd-orchestrator/phases/production-readiness-integration/library-candidates.json`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. An accepted release matrix names required profiles, clients, source schemas/keys/tenants, historical/delete semantics, scale/retention/RTO/RPO, ASO budgets and profile-specific authorization lifetime endpoints.

B. Pinned API, compatibility and maintenance/security evidence exists for each retained dependency; unsupported choices trigger a revised proposal, not invented feasibility. Reconcile Assess provenance finding through focused independent review.

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
