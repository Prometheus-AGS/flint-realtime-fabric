# Design — pri-c004-deployment-profiles

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve Q2 and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Render supported full and restricted-shape profiles from pinned artifacts and explicit identity/media/secret inputs. Remove absolute developer paths and false-success health checks; reject endpoints outside profile authority.

Owner: Fabric deployment + Gate. Legacy package: C02.
Dependencies: pri-c001-build-policy, pri-c002-local-fixtures, pri-c003-scope-research-contract.
Recommended agent: Codex; operator resolves recorded decisions.
Est. complexity: M; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `compose.yml`
- `.env.example`
- `k8s/overlays/ssr/`
- `crates/frf-gateway/src/config/mod.rs`
- `docs/ENVIRONMENT.md`

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. Clean local installation of each selected profile reaches semantic readiness with real dependency handshakes; missing issuer/keys/broker/auth authority fails visibly.

B. Pinned images, declared secret sources, exposed ports and feature flags match the profile; release artifacts contain no dev authorization bypass. Each profile declares TLS termination, certificate/key/trust sources and external/internal network boundaries; valid HTTPS succeeds, invalid or missing TLS inputs fail startup/readiness, and prohibited plaintext or backend-bypass access is denied.

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
