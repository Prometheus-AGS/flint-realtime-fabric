# Make selected deployment profiles portable and truthful

## Why

Assessment and analysis for production-readiness-integration identify an unmet
integration gate in C02. This change supplies one bounded
acceptance slice; implementation and production qualification remain separate.

## What Changes

Render supported full and restricted-shape profiles from pinned artifacts and explicit identity/media/secret inputs. Remove absolute developer paths and false-success health checks; reject endpoints outside profile authority.

## Ownership and Dependencies

Owner: Fabric deployment + Gate. Depends on: pri-c001-build-policy, pri-c002-local-fixtures, pri-c003-scope-research-contract.
Decision inputs before product implementation: Q2.
Status: proposal only; all implementation tasks are unchecked.

## Impact

Affected modules and repository-relative paths:
- `compose.yml`
- `.env.example`
- `k8s/overlays/ssr/`
- `crates/frf-gateway/src/config/mod.rs`
- `docs/ENVIRONMENT.md`

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
