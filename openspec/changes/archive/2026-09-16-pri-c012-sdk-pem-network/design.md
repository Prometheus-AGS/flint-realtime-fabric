# Design — pri-c012-sdk-pem-network

## Status and boundaries

Planning artifact, not a frozen implementation contract. Read the phase plan,
assessment, analysis and the owning repository's AGENTS.md/CLAUDE.md first.
Resolve none beyond dependencies and completed dependency receipts before product implementation.
The c003 research change may start immediately; its Q1–Q7 answers and focused
review acceptance are closure outputs, never prerequisites for starting research.

## Responsibility

Deliver one user-visible loop:

1. A client mutation updates its PGlite table optimistically.
2. The transport submits the mutation to Forge's authenticated REST or GraphQL API.
3. Forge commits the change to PostgreSQL under the user's RLS context.
4. Electric reads the committed row; Fabric's shape facade authorizes and proxies it.
5. PGlite electric sync applies the canonical row and PEM observes the local change.

ElectricSQL is downstream replication. It is not an upstream PGlite write API;
Forge owns that boundary.

Owner: Forge + Fabric shape facade + PGlite/PEM adapter. Legacy package: C07.
Dependencies: pri-c004-deployment-profiles, pri-c005-authority-lifetime and the implemented c011 tasks 2–3. The deferred c011 combined-campaign receipt is not an implementation dependency.
Recommended agent: Codex.
Est. complexity: L; Complexity score: High.
Model class: frontier (project model policy is absent).

## Affected paths

- `crates/frf-shape-electric/`
- `crates/frf-gateway/src/routes/shape.rs`
- `sdks/ts/`
- `../prometheus-entity-sync/packages/entity-sync-pglite/`
- `../flint-forge/crates/fdb-gateway/`
- runnable local compose/example assets in the owning repositories

Paths beginning ../ refer to sibling checkouts from the Fabric root. Absolute
ASO paths identify its separate checkout. Source changes and commits belong to
that repository; attach its exact revision/diff and verification receipts here.
Do not commit pre-existing foreign changes. Serialize shared gateway/config edits.

## Acceptance design

A. From the runnable prototype, one create/update/delete initiated against PGlite reaches PostgreSQL through Forge and returns through Electric/Fabric to the same PGlite table and PEM entity/list without manual refresh or direct database access.

B. A second authenticated client observes the canonical database change, tenant and field projection is enforced, and selecting LISTEN keeps the Forge GraphQL lane usable while shape sync remains explicitly gated.

## Compatibility and rollout

Keep proto/flint/v1 unchanged. Retain LISTEN as the Forge rollback source and
keep the last runnable prototype profile available during c013. Record forward
and rollback behavior for touched config/data/package boundaries before rollout.
If research demonstrates infeasibility, split/revise the proposal and preserve
its open acceptance obligation; diagnosis alone cannot complete implementation.

## Evidence and local verification

Implement the whole c012 and c013 user flow before testing it. Then use one
isolated local full-stack campaign for c011–c013 with synthetic records and real dependencies. CI only
builds/lints/typechecks/formats/packages. Missing prerequisites, skipped required
cases, empty streams and loopback substitution are failures. Capture
source/dirty-diff, locks, generated/package/image/config digests, topology,
commands, times, exits, expected scenarios and output hashes in a source-bound
receipt at the phase evidence path for this change. Documentation-only decisions
use recorded source evidence and operator provenance rather than invented tests.

## Library reuse

Reuse the existing PGlite/PEM transport seams and pinned generated SDK pipeline.
Package breadth is deferred until the runnable prototype works. Candidate details
remain in `library-candidates.json`; this revision does not select a new library.

<!-- Prior candidate evidence retained in the phase research artifact. -->

<!--

```json
[
  {
    "id": "cand-004",
    "name": "Pinned Connect-ES 1.x and existing generated SDK pipeline",
    "kind": "library",
    "repo_url": "https://github.com/connectrpc/connect-es",
    "registry": "npm",
    "fit_for_gap": "F08/F03: generated v2 watch transport and real packaged consumer proof",
    "verdict": "adapt",
    "license": "Apache-2.0",
    "maintenance": {
      "last_release": "2.2.0 (registry latest; not selected)",
      "registry_modified": "2026-09-07T06:45:44.581Z"
    },
    "evidence": [
      {
        "tier": 1,
        "source_url": "https://github.com/connectrpc/connect-es",
        "claim": "Existing TypeScript RPC client family found."
      },
      {
        "tier": 2,
        "source_url": "https://github.com/connectrpc/connect-es/blob/main/MIGRATING.md",
        "claim": "Current docs show v2 changes to client construction and Protobuf-ES generation; they cannot be copied into the pinned v1 SDK."
      },
      {
        "tier": 3,
        "source_url": "https://registry.npmjs.org/@connectrpc%2Fconnect/1.6.1",
        "claim": "Pinned-line 1.6.1 metadata is available; latest lookup separately returns 2.2.0."
      }
    ],
    "risks": [
      "Exact v1 browser streaming/cancellation behavior needs installed-source inspection and packed network tests; current Context7 examples are mostly v2.",
      "CJS export is advertised without an emitted CJS artifact; fix build/export contract before compatibility signoff.",
      "RPC proto v2 does not require Connect-ES package v2. ADR-003 upgrades require an explicit superseding decision."
    ],
    "decision_rationale": "Retain ADR-003 and generated clients. Repair distribution and adapters using the current toolchain; avoid an unrelated major-version migration."
  }
]
```
-->
