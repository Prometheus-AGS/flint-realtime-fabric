# Design — pri-c013-sync-persistence

## Status and boundaries

Implementation complete in source; the combined c011–c013 runtime campaign and
evidence remain. Read the phase plan, assessment, analysis and the owning
repository's AGENTS.md/CLAUDE.md first.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Extend the working c012 transport with an outbox stored in PGlite. Each local
mutation receives a stable client mutation ID and records its operation and
expected entity version. A dispatcher submits pending entries through Forge,
persists retry/error state, and removes an entry only after the canonical row or
delete is observed through Electric/Fabric. Forge must apply the mutation ID
idempotently or expose a conflict that the client can reconcile explicitly.

The implemented boundary uses a UUID `Idempotency-Key`. Forge serializes
concurrent attempts with a transaction-scoped advisory lock and commits the
target mutation plus `flint.mutation_receipts` response in one RLS transaction.
The PGlite outbox persists the optimistic row and queued operation atomically,
dispatches one outstanding operation per entity, and retains it through the
`sent` state until the Electric shape stream observes matching canonical state.
Rejected operations remain visible with retry and discard controls; later
queued operations are rebased when an earlier canonical result or discard
changes their baseline.

Owner: PGlite/PEM sync transport + Forge mutation API. Legacy package: C10 / F10.
Dependencies: functional completion of c012 tasks 2–3. Its deferred combined-campaign receipt is not an implementation dependency.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `../prometheus-entity-sync/packages/entity-sync-pglite/`
- `../prometheus-entity-sync/packages/entity-sync-core/`
- `../flint-forge/crates/fdb-gateway/`
- `../flint-forge/crates/fdb-app/`
- runnable local compose/example assets in the owning repositories

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. A mutation created while Forge/Electric is unavailable survives client restart, is submitted when connectivity returns, and converges through the canonical Electric/Fabric row in PGlite and PEM.

B. Retries do not create duplicate database effects; server rejection or version conflict is visible and recoverable; two clients converge after create/update/delete while tenant isolation remains enforced.

## Compatibility and rollout

Keep the c012 online path runnable while adding durability. Schema additions to
PGlite must be idempotent and restart-safe; Forge idempotency metadata must not
change the canonical entity contract. Retain LISTEN as the Forge rollback source.
If research demonstrates infeasibility, split/revise the proposal and preserve
its open acceptance obligation; diagnosis alone cannot complete implementation.

## Evidence and local verification

Finish the offline/restart behavior before testing it. Then run the one combined
c011–c013 local full-stack campaign with real dependencies, including client and Forge restarts.
CI only builds/lints/typechecks/formats/packages. Missing prerequisites, skipped
required cases, empty streams and loopback substitution are failures. Capture
source/dirty-diff, locks, generated/package/image/config digests, topology,
commands, times, exits, expected scenarios and output hashes in a source-bound
receipt at the phase evidence path for this change. Documentation-only decisions
use recorded source evidence and operator provenance rather than invented tests.

## Library reuse

No new library is selected here. Existing dependencies require c003 qualification where applicable; this proposal is not evidence that unresearched APIs fit.
