# Refinement log — pri-c006-watch-contract

## Iteration 1

Result: **DETERMINISTIC PASS; REVIEW PENDING**.

| Constraint | Result | Evidence |
|---|---|---|
| A | PASS | Fixture checker validates six current v1 files, three required key shapes, four resume cases and the transaction/barrier/auth/delete/lag/cancel matrix. |
| B | PASS | Buf FILE compatibility passes; five prost tests generate and encode v1 and v2 together; gateway and Rust SDK compile. |
| C | PASS | Consumer acceptance was recorded from exact clean Forge/PEM revisions before any c006 proto edit. |
| D | PASS | ADR-010 and acceptance B fix the pre-1.0 minor-version and dedicated-port decisions. |
| P1 | PASS | Focused Rust, Buf, Node, format, strict Clippy and dependent compile checks pass; final OpenSpec/size/diff checks remain in task 5. |

The historical `proto-v1` tag differs from the current v1 source because later
work changed generation metadata and the agent stream. The baseline states that
fact. c006 freezes and verifies the current accepted v1 hashes rather than
inventing tag equality or rewriting history.

The next step is the post-QA independent adversarial review. Convergence remains
open until all critical and warning findings are resolved.

Before review dispatch, the contract was tightened so every mutation repeats
its entity type and tenant. Authorized delete invalidations are now
self-describing when persisted without the stream-admission frame; the fixture,
generated-code test and deterministic checker enforce this requirement.

## Iteration 2

Initial independent review blocked on the still-open final task. After review,
the full current local gate set passed, the source-bound receipt was refreshed,
and task 5 was completed. The required finding is resolved; a fresh resolution
review remains the archive gate.

## Iteration 3

The first resolution review required the full workspace R1/R2 gate family.
Workspace check, library/binary Clippy, `frf-gateway/dev-endpoints` Clippy and
test-target Clippy all pass locally with warnings denied. The receipt records
the exact commands; a second fresh resolution review remains pending.

## Iteration 4

The second fresh resolution review passed with 0 critical findings, 0 warnings
and 0 suggestions; its strict sycophancy screen also passed. Every blocking
constraint is satisfied and the artifact converged.
