# Refinement log — pri-c007-broker-replay

## Iteration 1

Result: **DETERMINISTIC PASS; REVIEW PENDING**.

| Constraint | Result | Evidence |
|---|---|---|
| A | PASS | Live crash-before-checkpoint replay and restart-after-checkpoint recovery passed at offsets 1→2. |
| B | PASS | Delivered broker positions ignore repeated source offsets; stable message-ID deduplication passed before and after server restart. |
| C | PASS | Independent named consumers, partition 1, physical expiry/resnapshot and receiver cancellation passed. |
| D | PASS | Receipt binds exact source and matching Iggy image; rendered server config and logs confirm the actual settings and owned cleanup. |
| P1 | PASS | Focused and workspace Rust gates, format, shell, Compose, file-size, diff and strict OpenSpec checks passed locally. |

An initial restart check exposed Docker's post-restart host-port readiness and a
zero-offset persistence ambiguity. The runner now re-resolves the published port
and the durable checkpoint scenario uses a nonzero position. A first expiry
attempt also exposed a stale environment-variable name and the empty-partition
floor edge case. The corrected fixture enables the actual data-maintenance
cleaner, closes 1 KB segments, observes their deletion and requires the adapter's
resnapshot error.

Independent adversarial review remains the convergence gate.

## Iteration 2

The initial review blocked on one substantive defect and three packet-scope
defects. The real defect was the provisional source offset returned from
`publish`, which remained unsafe as a replay cursor. The adapter now maps the
stable accepted message ID back to its exact broker position and verifies that
mapping across sequential, retry, restart and concurrent publishers. An explicit
change file list plus exact staging prevents unrelated workspace edits and
untracked evidence from contaminating the resolution packet. All deterministic
gates pass again; fresh resolution review remains pending.

## Iteration 3

The first scoped resolution review found a retention race after the preflight
floor check. An explicit subscription now validates its first delivered offset
against the requested position and emits `resnapshot_required:` on any advance.
The proposal status was also refreshed. Ten unit tests, the owned live run and
the complete workspace gate matrix pass; second resolution review remains.

## Iteration 4

The second scoped resolution review found that restart persistence was seeded
through an administrative seek rather than the adapter's public acknowledgement
path, so the evidence did not yet prove the required production route. The seed
scenario now calls `LogBroker::ack` at broker offset `1`. A new owned run proves
that exact checkpoint survives a real Iggy restart and resumes at offset `2`,
then appends at `3` with stable deduplication. The implementation status and all
source-bound evidence now describe the completed acceptance run. All local gates
pass; fourth independent review remains.

## Iteration 5

Fresh-context, cross-model review passed with no findings. The judge verified
the acceptance criteria, offset and commit semantics, retention race handling,
restart runner, cancellation, public port stability, security constraints and
failure propagation. Strict sycophancy screening also passed. The artifact is
converged and ready to archive.
