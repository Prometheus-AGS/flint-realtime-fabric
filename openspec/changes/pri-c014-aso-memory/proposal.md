# Diagnose and reduce the existing ASO replica memory cost

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C08. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Reconcile current source with mounted receipts and profile the existing 16203-row workload. Apply measured, bounded memory corrections while preserving SQL/checkpoint atomicity, owner fencing and the experimental gate.

## Ownership and Dependencies

Owner: ASO materializer + PEM. Depends on: pri-c002-local-fixtures, pri-c003-scope-research-contract.
Decision inputs before product implementation: Q7.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/`
- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/scripts/test-ra11c-materialization.py`

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
