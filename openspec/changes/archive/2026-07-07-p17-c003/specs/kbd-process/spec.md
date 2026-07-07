# kbd-process (delta)

## ADDED Requirements

### Requirement: Every code change MUST pass the per-change QA gate before archive

The execute loop MUST run the QA gate (`.kbd-orchestrator/bin/qa-gate.sh`) against each
code change after it reaches DONE and before archive, evaluating the BLOCKING subset of
`.kbd-orchestrator/constraints.md`. A change with a failing gate MUST be marked BLOCKED
and refined; only an ALL-PASS gate result unlocks archive.

#### Scenario: a change with a blocking violation cannot archive

- **WHEN** the QA gate runs against a change that violates a BLOCKING constraint
- **THEN** the gate exits non-zero and writes a BLOCKED verdict to the change's
  `.refiner/artifacts/<id>/refinement_log.md`
- **AND** the change is not archived until a re-run returns ALL PASS

#### Scenario: gate does not false-positive on test fixtures

- **WHEN** the gate's secret check runs over a tree containing a test-fixture private key
- **THEN** the fixture under `tests/fixtures/` is not flagged as a hardcoded secret
- **AND** a real secret literal in production source IS flagged
