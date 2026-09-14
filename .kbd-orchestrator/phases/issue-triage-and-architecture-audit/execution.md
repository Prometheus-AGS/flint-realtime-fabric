EXECUTION: issue-triage-and-architecture-audit
Project: flint-realtime-fabric
Date: 2026-09-14
Selected backend: openspec
Dispatched to: SELF (Claude Code) via `/kbd-apply`, task-by-task
Backend rationale: `openspec/` exists at the project root, `project.json` declares
  `"change_backend": "openspec"`, and `/kbd-apply detect` returns `openspec`. The phase
  needs spec-backed traceability: it closes three GitHub issues and repairs two BLOCKING
  constraint violations, so each change must be independently verifiable and archivable.
Backend entrypoint: `/kbd-apply <change-id>` — never bare `/opsx:apply`
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/issue-triage-and-architecture-audit/plan.md

## EXECUTION SCOPE

- p38-c001-iggy-handshake-diagnosis: diagnose why no available Iggy server completes a
  handshake with the pinned 0.6.203 client
- p38-c002-prove-channel-id-fix: run and sabotage the issue #2 guard against a working server
- p38-c003-e2e-tenant-uuid: fix the non-UUID `tenantId` the gateway always rejects
- p38-c004-gateway-main-size-and-enforcement: bring `main.rs` under the 500-line cap, add a check
- p38-c005-claude-md-adr-reconciliation: reconcile CLAUDE.md with the ADRs that settled it
- p38-c006-dead-code-and-guard-classification: delete orphaned `ws.rs`, classify 4 ignored tests

## DISPATCH CONTRACTS

**Concrete models cannot be resolved.** `project.json` has no `model_policy` block, so
`model_policy.registry.<class>.<active_environment>` does not exist. Per
`references/model-routing.md` ("If `model_policy` is absent from `project.json`, treat all
phases as `frontier`. Never silently downgrade"), the `Model class` values below are the
plan's routing *hints* for `/opsx:apply`; no concrete model name is asserted because none
can be resolved from config.

- p38-c001-iggy-handshake-diagnosis → SELF (Claude Code)
  Entry: `/kbd-apply p38-c001-iggy-handshake-diagnosis`
  Model class: frontier
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: cause is undetermined; protocol-level debugging with a stop-and-report
    branch. Scored High: >8 tasks' worth of investigation, cross-boundary, no prior art.
  Progress file: .kbd-orchestrator/phases/issue-triage-and-architecture-audit/progress.json
  Handoff: report completion by updating progress.json and committing

- p38-c002-prove-channel-id-fix → SELF (Claude Code)
  Entry: `/kbd-apply p38-c002-prove-channel-id-fix`
  Model class: small
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: 5 mechanical tasks, no new abstractions, single crate.
  Progress file: same
  Handoff: same

- p38-c003-e2e-tenant-uuid → SELF (Claude Code)
  Entry: `/kbd-apply p38-c003-e2e-tenant-uuid`
  Model class: small
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: 3 tasks, one shell script, direct analog exists.
  Progress file: same
  Handoff: same

- p38-c004-gateway-main-size-and-enforcement → SELF (Claude Code)
  Entry: `/kbd-apply p38-c004-gateway-main-size-and-enforcement`
  Model class: small
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: 6 tasks, mechanical extraction within one crate, no new abstraction.
  Progress file: same
  Handoff: same

- p38-c005-claude-md-adr-reconciliation → SELF (Claude Code)
  Entry: `/kbd-apply p38-c005-claude-md-adr-reconciliation`
  Model class: small
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: 5 tasks, documentation only, each correction cites a specific ADR.
  Progress file: same
  Handoff: same

- p38-c006-dead-code-and-guard-classification → SELF (Claude Code)
  Entry: `/kbd-apply p38-c006-dead-code-and-guard-classification`
  Model class: medium
  Concrete model: UNRESOLVABLE (no model_policy in project.json)
  Model rationale: 6 tasks crossing several crates; classification requires judgement about
    what a test actually guards.
  Progress file: same
  Handoff: same

## EXECUTION ROUND ORDER

```
Round 1 (parallel-safe): p38-c001, p38-c003, p38-c004, p38-c005
Round 2 (serial):        p38-c002   — hard dependency on c001
Round 3:                 p38-c006   — classification needs c001's outcome
```

c003/c004/c005 touch disjoint trees (a shell script, the gateway crate, root docs) and share
no files with c001.

## APPROVAL GATES

- **p38-c001 T4 — stop and report.** If the diagnosis concludes the remedy is repinning the
  *client* rather than the server, halt and report. That widens scope beyond this phase and
  is an operator decision, not an execution step.
- **p38-c002 T5 — closing issue #2** requires demonstrated receipt. Do not close on a green
  compile.

## FALLBACK CONDITIONS

- Backend fallback is not applicable: `openspec` is already the selected backend and the
  protocol's fallback target.
- If `/kbd-apply` cannot drive a change task-by-task, halt rather than invoking bare
  `/opsx:apply` — that is the seam that historically broke plan→execute.

## VERIFICATION REQUIREMENTS

Local only. CI may build, lint, typecheck, format and package; it must **never** run tests
(AGENTS.md:14-28, CLAUDE.md "Testing Policy — Local Integration Only").

```bash
cargo check --workspace
cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic
cargo fmt --check --all
cargo test --workspace                      # default suite; 4 tests are #[ignore]d
openspec validate --changes --json          # P1: every change must validate
```

Per-change verification is defined in each change's `tasks.md`.

## PROGRESS LEDGER

- [PENDING] p38-c001-iggy-handshake-diagnosis — SELF
- [PENDING] p38-c002-prove-channel-id-fix — SELF (blocked on c001)
- [PENDING] p38-c003-e2e-tenant-uuid — SELF
- [PENDING] p38-c004-gateway-main-size-and-enforcement — SELF
- [PENDING] p38-c005-claude-md-adr-reconciliation — SELF
- [PENDING] p38-c006-dead-code-and-guard-classification — SELF (blocked on c001)

## OUTPUTS

- Six OpenSpec changes created and validating (`openspec validate --changes` → 6/6 pass)
- This execution contract

## BLOCKERS

- **p38-c002 and p38-c006 are blocked on p38-c001.** Not a defect — a stated dependency.
- **No blocker on starting.** c001, c003, c004 and c005 are all startable now.

## DEPARTURES FROM PROTOCOL — recorded, not silent

1. **F3 plan/execute boundary.** The execute protocol states: *"`/kbd-plan` creates the
   change (`/opsx:new`); `/kbd-execute` drives it via `/kbd-apply`. Do not re-create changes
   here."* The six changes did not exist at execute time, because the plan stage's step 10
   was deliberately deferred pending an operator decision about the OpenSpec ledger (which
   was in a state where `openspec validate` failed for all 126 active changes). They were
   created here, at operator direction, using `openspec new change`. `/opsx:new` itself was
   not invocable — it exists only as a plugin command under an unrelated marketplace.

2. **Runtime-authority steps 7–8 do not apply.** `prometheus` 1.8.0 is on PATH, but
   `kbd_runtime_authoritative` returns false for this repo, so file-based projections are
   canonical. No `prometheus kbd change|task` registration was performed; claiming otherwise
   would be false.

3. **Adversarial review has not run for any artifact in this phase.** The model gateway at
   `http://localhost:4000/v1` returns HTTP `401` (reachable, missing Authorization header —
   CORRECTED from an earlier false `000`/unreachable claim); `model-preflight.json` reports `status:
   "ok"` from a stale 24 h cache, and `distinct_models: 2` cannot fill three roles. Per the
   skill's "never block the stage on preflight status", stages proceeded. **No CRITICAL
   findings were generated for the assessment, the plan, or this contract** — downstream
   must not read that silence as endorsement.

4. **`skip_specs: true` on all six changes.** Applied per the rule "deltas where behaviour
   changes". c001 is diagnostic (its output is a finding; the behaviour change belongs to
   whatever fix follows), c002 proves an already-landed behaviour, and c003–c006 are a shell
   fix, a file split, docs, and dead-code removal. The `event-spine` capability delta is
   **owed by the eventual handshake fix**, not by this phase. Note no archived change has
   ever carried a broker/publish-subscribe delta; all 29 capabilities were generated at
   archive time, so creating `event-spine` is ordinary practice when that fix lands.

## REFLECTION HANDOFF

`/kbd-reflect` should consume:

- Whether c001 determined the handshake mechanism, or hit its stop-and-report branch.
- Whether the c002 guard was observed to **fail** under sabotage. If it was not, the issue
  #2 fix remains unproven regardless of what else completed — that is the single most
  important signal in this phase.
- Which of the four `#[ignore]`d tests c006 reclassified as runnable, and which of those
  actually ran.
- Whether the file-size check added by c004 was itself sabotage-tested.
- The out-of-plan ledger repair recorded in `progress.json → notes` and commit `4aa9663`,
  including the 46 latent `validate --archived` failures tracked as issue #8.

EXECUTION READY
