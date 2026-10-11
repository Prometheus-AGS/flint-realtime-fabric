# EXECUTION: production-readiness-integration

Project: flint-realtime-fabric
Date: 2026-09-15
Selected backend: openspec
Dispatched to: Codex through kbd-apply
Backend rationale: OpenSpec changes already exist and the phase requires
cross-repository, spec-backed task and evidence traceability.
Backend entrypoint: /kbd-apply, one task at a time
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/production-readiness-integration/plan.md

## Execution scope

The canonical order and dependency graph are in planned-changes.json revision 3.
The active critical path is c011 Forge realtime, c012 the runnable
PGlite–Forge–Electric–PEM loop, then c013 durable outbound replay. ASO, media,
federation, admin, platform parity, operations and release signoff follow that
usable prototype. Fabric CRDT persistence is deferred to a later phase.

## Dispatch contracts

Every change is driven by Codex through the OpenSpec-aware kbd-apply task loop.
Each change reads its proposal.md, design.md, tasks.md and spec delta before
implementation. Model class is frontier because project.json has no model policy;
each proposal still records its independent complexity score.

For each change:

1. Read current-waypoint.json and the change artifacts.
2. Use kbd-apply begin-task before exactly one task.
3. Implement the complete user-facing behavior owned by that task.
4. Use kbd-apply end-task to synchronize typed task state and projections.
5. After the whole slice is functional, run its single local full-stack
   integration campaign, constraint QA, diff review, OpenSpec verification and archive.
6. Preserve evidence, certification and publication as independent dimensions.
7. Commit only phase-owned changes in the repository that owns them.

## Approval gates and resolved defaults

The operator authorized autonomous execution through reflection, commits, pushes
and PR creation across the named repositories. This also authorizes the typed KBD
state initialization required by kbd-execute. A reversible pre-migration archive
was created at /tmp/frf-kbd-backup.c6pcfK/kbd-orchestrator.tar with SHA-256
947ee14e8d691677f3d49ebca26f5b7c9fd8a5ddb780ffcf941908ff7d9c6e22.

Until superseded by operator input, c003 uses the recommended conservative
release contract:

- First release: Fabric, Gate, Forge, PEM and ASO data integration. Media,
  federation, admin and additional SDKs retain separate phase gates.
- Qualify the existing ASO restricted-shape deployment and a standalone full
  Fabric profile separately.
- First required clients: TypeScript browser/PEM, Node ESM/CJS and Rust.
- Durable at-least-once changes within a bounded retention window, followed by
  explicit resnapshot when history expires.
- Preserve ASO's 512 MiB incremental RSS ceiling and five-second
  server-produced-frame revocation boundary; qualify browser first.
- Start capacity validation at 1,000 subscriptions, 100 committed changes/s for
  60 minutes, p95 <= 1 s and p99 <= 3 s. The source schema/tenant inventory and
  measured recovery behavior must set final retention/RTO/RPO before signoff.

Database key/tenant/delete behavior remains evidence-derived: support the actual
consumer schemas found in c003, fail enrollment for unsupported mappings, and
deliver authorized invalidations for deletes unless an audit consumer and policy
explicitly require historical row contents.

For the prototype path, ElectricSQL is downstream database replication. Upstream
PGlite mutations use Forge's authenticated mutation API; a PGlite outbox adds
durable retry and reconciliation in c013. This is the supported interpretation
of the requested PGlite -> ElectricSQL -> database sync loop.

## Fallback conditions

- If a task cannot fit one bounded session, split its OpenSpec change without
  weakening the parent acceptance criteria.
- If a pinned dependency cannot meet its accepted contract, return to c003,
  evaluate alternatives and revise the dependent design before implementation.
- If sibling worktree ownership overlaps active uncommitted work, create an
  isolated codex/production-readiness-integration branch or worktree and merge
  by commits; do not overwrite user work.
- A missing local service blocks only its dependent runtime proof. Continue
  independent implementation, build, static and research work.

## Verification requirements

- Tests run only on local composed infrastructure. CI may build, lint, typecheck,
  format and package; no CI test dispatch is permitted.
- c011-c013 receive no isolated acceptance-test campaign before their user flow
  exists. Each passes one meaningful local full-stack campaign plus applicable
  build/lint/format, OpenSpec and review gates before archive.
- Release evidence binds exact source/diff, lockfiles, generated artifacts,
  packages/images, effective configuration, topology, commands and output hashes.
- The pre-existing Fabric shape edits named in entry-state.json remain untouched
  unless a later ASO-owned change explicitly reconciles and adopts them.

## Progress ledger

Ten changes are complete. c011 task 2 is next, followed by c012 and c013. Product
changes obey planned-changes.json revision 3 dependencies.

## Outputs

- Per-change source and evidence in the owning repository
- OpenSpec verification and archive records
- Cross-project commit and PR references
- Current-source release verdicts per capability
- Phase reflection and final KBD handoff

## Reflection handoff

kbd-reflect consumes progress.json, archived OpenSpec changes, local test and
review receipts, source/commit/PR manifest, unresolved capability gates and the
final per-profile release verdict. Reflection does not convert unsupported or
unpublished capabilities into completion.

EXECUTION READY
