# Resolve release decisions and complete dependency qualification research

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C01 + Analyze backlog. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Resolve Q1–Q7 with provenance and research the named identity/policy, persistence/deployment, media/federation and platform backlog in bounded passes. Inventory source/package/image revisions and runtime profiles. Record Q8 separately; KBD migration is not part of this change. Research and decision facilitation may start immediately; unresolved Q1–Q7 block closure and dependent implementation, not the start of this change.

## Ownership and Dependencies

Owner: Fabric coordinator + Gate/Forge/PEM/ASO owners. Depends on: none.
Decision inputs before product implementation: none beyond dependencies.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `.kbd-orchestrator/phases/production-readiness-integration/analysis.md`
- `.kbd-orchestrator/phases/production-readiness-integration/decision-log.md`
- `.kbd-orchestrator/phases/production-readiness-integration/library-candidates.json`

Dependency-rule impact: keep domain/application imports inward; adapter behavior
is accessed through ports, with one port per adapter and gateway composition.
Changes to public domain/port types require a documented semver decision.

## Foundation Compatibility

Preserve the completed Phase 0 foundation: workspace build and immutable v1
contract. This is a readiness follow-up; it does not recreate or claim Phase 0.

## Non-goals

Unrelated refactoring, automatic production deployment/publication, and KBD
runtime migration are outside this proposal. Full readiness requires the phase's
other accepted gates; this slice cannot certify them.
