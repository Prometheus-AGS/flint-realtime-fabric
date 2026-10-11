# Acceptance A — transactional recovery and session fencing

Result: **Passed**

## Crash and recovery

The current-source mounted materializer passed all thirteen durability and
ownership checks on the 16,203-row protected dataset. It proved that an
interruption before commit leaves neither partial rows nor a checkpoint, a
failure during materialization rolls back the row and checkpoint together, and
a restart after SQL commit but before PEM publication reconstructs the graph
from durable authorized state without skipping the committed revision.

Disposed owners could not publish a pending revision. The replacement owner
applied that revision coherently to SQL and PEM, and authority loss emitted a
revalidation failure while retaining the last authorized SQL, checkpoint and
graph state.

## Mounted logout and practice switch

The shipping `SessionProvider` and `GraphProvider` were mounted in Chrome with
two independent Kratos sessions and two populated practices. Both graphs
contained the expected seven projections. Practice switch and logout removed
the protected view synchronously, the old graph was absent before replacement,
no successful old-scope response completed after either fence, and the
replacement session was revoked through the real Kratos admin endpoint. Its
subsequent Gate request returned HTTP 401.

All synthetic identities, relational rows, PGlite state, helper state and
service changes were removed or restored by the campaign cleanup.

## Source-bound evidence

| Artifact | SHA-256 |
|---|---|
| `campaign.json` | `6abf86af9336ece7e2fd32a6229f85d039e9b43b4cc20ff0415aa00cb00bea51` |
| `task-4-mounted-materializer.json` | `3295c7d983f9f16e8f78dc4e04500c0371c2758ded8729c3184423926ee69f93` |
| `task-4-mounted-browser-memory.json` | `9b08419b97cb587f9ad42a26e86dc18d0e390a3d9b8551a8a1683ef9103ab532` |

The campaign reports unchanged selected source for ASO, Fabric and Gate between
its before and after snapshots.
