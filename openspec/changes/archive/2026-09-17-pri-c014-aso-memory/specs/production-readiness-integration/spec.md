## ADDED Requirements

### Requirement: Diagnose and reduce the existing ASO replica memory cost

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c014-aso-memory` are satisfied and its accepted local verification is executed
- **THEN** Same-fixture browser campaign measures incremental RSS at or below 536870912 bytes and heap at or below 268435456 bytes, unless c003 records an explicit operator-approved replacement contract.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Worker/DB/graph-copy and cold-fold allocations explain the result; passing behavior survives optimization. If one bounded session cannot close the gate, record findings and split remaining implementation before continuing; do not close this change on diagnosis alone. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
