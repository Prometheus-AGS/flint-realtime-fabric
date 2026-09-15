# Decision log — production-readiness-integration

## 2026-09-15 — Analyze: preserve established stack

Decision: adapt existing Iggy, pg_walstream, PGlite/materializer and Connect-ES
integration. No replacement dependency or major upgrade selected.
Provenance: repository constraints, installed source and bounded research.
Candidate references: cand-001 through cand-004 in library-candidates.json.
Revisit if the pinned implementation demonstrably cannot meet an accepted contract.

## 2026-09-15 — Analyze: distinguish recovery positions

Decision: specification must distinguish source LSN, durable broker position,
event identity and consumer checkpoint; explicit acknowledgement and transaction
boundaries need proof. Current resettable envelope counters are insufficient.
Provenance: pinned broker and CDC source inspection; not a runtime failure receipt.

## 2026-09-15 — Analyze: preserve ASO authority and qualification boundaries

Decision: reuse the materializer and its existing adoption gate. Preserve the
512 MiB RSS limit and existing server-produced-frame revocation boundary unless
the operator explicitly changes requirements. Browser/native scope is pending.
Provenance: existing ASO contract and assessed receipts; Q7 requested.

## 2026-09-15 — Operator clarification requested

Elicitation IDs: Q1–Q8 in analysis.md; channel: task question UI.
Status: pending; suggested choices are not user decisions.
Dependent specification work remains open. No answer is inferred from elapsed time.
Legacy authority is unchanged pending Q8; no implementation changes are registered.

## 2026-09-15 — Review correction: research coverage

The four candidates cover the data path only (F01/F03/F07/F08). Other phase
goals retain a named research backlog and cannot be fully planned from this
artifact. WAL adoption is conditional on maintenance/security verification.
Provenance: isolated Analyze reviewer round 1, one critical and one warning.

## 2026-09-15 — Analyze review outcome

Second review: PASS with two warnings. Final corrections avoid ruling out
unevaluated CDC services and align machine gap identifiers with bounded scope.
The reviewed snapshots are retained; final warning edits were not re-vetted.
Q1–Q8 and the broader research backlog remain pending.

## 2026-09-15 — Plan revision 2

Created 23 proposal slices from the assessment and bounded analysis. Q1–Q7
remain closure outputs of c003; no suggested answer was accepted. c016 final
operations qualification follows all selected implementation/client dependencies.
Tonic Rust client generation is owned by c010 before Forge c011.
Two Plan review rounds ended BLOCK; final corrections are recorded but not
independently re-vetted. Preserve that status until focused review accepts them.
No source execution, package publication or KBD runtime migration occurred.

## 2026-09-15 — c003 conservative release contract accepted

The autonomous execution authorization in `execution.md` delegates the stated
conservative defaults until superseded. Q1–Q7 are resolved by
`evidence/pri-c003-scope-research-contract/release-contract.md`; Q8 leaves legacy
KBD authority unchanged.

The first release target is the Fabric/Gate/Forge/PEM/ASO data path. Standalone
`full` and ASO `shape-only` profiles qualify separately. Required clients are
browser/PEM, Node ESM/CJS and Rust. The accepted replay, schema/key/tenant,
delete, capacity, latency, retention, RTO/RPO, memory and revocation boundaries
are frozen in the release contract.

## 2026-09-15 — c003 dependency proposal revised after current audit

Pinned architectural dependencies remain the starting point, but current
production qualification is blocked. `cargo audit` against RustSec database
commit `e2e640471715167f73e22eaf761f2e547adafeec` reports eight vulnerabilities,
seven unmaintained warnings, four unsound warnings and two yanked packages.
The Iggy, SurrealDB, Loro and hosted-media closures contain release-relevant
findings; pg_walstream 0.6.3 is behind the active 0.8 line; mutable server image
tags also fail identity requirements.

Decision: keep the existing adapter architecture, require exact compatible
version/digest selection and local behavior proof in c004/c005/c007/c008/c013/
c017/c020, and require a clean applicable audit gate in c016/c023. No library
upgrade is inferred by this research-only change.

Cross-project scans also block the current integrated release: Gate reports 10
Rust vulnerabilities, Forge 6, PEM's production monorepo graph includes 2
critical npm advisories, and ASO web includes 13 high/moderate React Router
advisories. `integration-audit-summary.json` assigns remediation/non-applicability
proof to the owning changes and requires fresh per-repository evidence at c023.
